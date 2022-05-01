pub mod state;

use crate::source::UpdateInfo;
use log::{info, warn};
use lsp_server::{self, Connection, Message};
use lsp_types::{
    self,
    notification::{Cancel, DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
};
use state::State;

pub async fn run() {
    let (connection, io_threads) = Connection::stdio();

    let mut text_document_sync_options = TextDocumentSyncOptions::default();
    text_document_sync_options.open_close = Some(true);
    text_document_sync_options.change = Some(TextDocumentSyncKind::INCREMENTAL);
    text_document_sync_options.save = Some(TextDocumentSyncSaveOptions::Supported(true));

    let mut server_capabilities = ServerCapabilities::default();
    server_capabilities.text_document_sync = Some(TextDocumentSyncCapability::Options(
        text_document_sync_options,
    ));
    let server_capabilities = serde_json::to_value(server_capabilities)
        .expect("failed to convert server capabilities to json value..");

    let initialization_params: InitializeParams = serde_json::from_value(
        connection
            .initialize(server_capabilities)
            .expect("failed to initialize.."),
    )
    .expect("failed to convert json value to initialization_params..");

    let root_dir = initialization_params
        .root_uri
        .unwrap()
        .to_file_path()
        .unwrap();

    info!(
        "starting bokuteki language server at {}",
        root_dir.display()
    );

    let mut state = State::default();
    for msg in &connection.receiver {
        if let Message::Request(request) = &msg {
            if connection.handle_shutdown(request).unwrap() {
                break;
            }
        }
        handle_msg(&mut state, msg);
    }
    io_threads.join().unwrap();
}

fn handle_msg(state: &mut State, msg: Message) {
    match msg {
        Message::Notification(notification) => {
            if let NotificationConverter(Some(notification)) =
                NotificationConverter::new(notification)
                    .convert_and_then::<Cancel>(state, handle_cancel_notification)
                    .convert_and_then::<DidChangeTextDocument>(
                        state,
                        handle_did_change_text_document_notification,
                    )
                    .convert_and_then::<DidOpenTextDocument>(
                        state,
                        handle_did_open_text_document_notification,
                    )
                    .convert_and_then::<DidSaveTextDocument>(
                        state,
                        handle_did_save_text_document_notification,
                    )
            {
                warn!("unhandled message: {:?}", notification);
            }
        }
        msg => {
            warn!("unhandled message: {:?}", msg);
        }
    }
}

use lsp_types::notification::Notification as LspTypesNotification;
fn handle_cancel_notification(
    _state: &mut State,
    cancel: <Cancel as LspTypesNotification>::Params,
) {
    warn!(
        "request {:?} was cenceled, but currently cancellation is not implemented",
        cancel.id
    )
}

fn handle_did_open_text_document_notification(
    state: &mut State,
    did_open_text_document: <DidOpenTextDocument as LspTypesNotification>::Params,
) {
    let url = did_open_text_document.text_document.uri;
    let src = did_open_text_document.text_document.text;
    let (errors, warnings) = state.add_source(url, src);

    for error in errors {
        warn!("parse error: {}", error);
    }
    for warning in warnings {
        warn!("parse warning: {}", warning);
    }
}

fn handle_did_change_text_document_notification(
    state: &mut State,
    did_change_text_document: <DidChangeTextDocument as LspTypesNotification>::Params,
) {
    let url = did_change_text_document.text_document.uri;
    let update_infos = did_change_text_document
        .content_changes
        .into_iter()
        .map(|change_event| {
            let range = change_event.range.expect("must have range");
            UpdateInfo {
                start: (range.start.line, range.start.character),
                end: (range.end.line, range.end.character),
                text: change_event.text,
            }
        })
        .collect();
    let (errors, warnings) = state.update(url, update_infos);
    for error in errors {
        warn!("parse error: {}", error);
    }
    for warning in warnings {
        warn!("parse warning: {}", warning);
    }
}

fn handle_did_save_text_document_notification(
    _state: &mut State,
    _did_save_text_document: <DidSaveTextDocument as LspTypesNotification>::Params,
) {
    // do nothing
}

struct NotificationConverter(Option<lsp_server::Notification>);

impl NotificationConverter {
    fn new(notification: lsp_server::Notification) -> Self {
        Self(Some(notification))
    }

    fn convert_and_then<T: lsp_types::notification::Notification>(
        self,
        state: &mut State,
        action: fn(&mut State, T::Params),
    ) -> Self {
        if let Some(notification) = self.0 {
            match notification.extract(T::METHOD) {
                Ok(n) => {
                    action(state, n);
                    NotificationConverter(None)
                }
                Err(notification) => NotificationConverter(Some(notification)),
            }
        } else {
            NotificationConverter(None)
        }
    }
}

use crate::compile;
use log::{info, warn};
use lsp_server::{self, Connection, Message};
use lsp_types::{
    self,
    notification::{Cancel, DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
};

pub async fn run() {
    let (connection, io_threads) = Connection::stdio();

    let mut text_document_sync_options = TextDocumentSyncOptions::default();
    text_document_sync_options.open_close = Some(true);
    text_document_sync_options.change = Some(TextDocumentSyncKind::FULL);
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

    for msg in &connection.receiver {
        if let Message::Request(request) = &msg {
            if connection.handle_shutdown(request).unwrap() {
                break;
            }
        }
        handle_msg(msg);
    }
    io_threads.join().unwrap();
}

fn handle_msg(msg: Message) {
    match msg {
        Message::Notification(notification) => {
            if let Hoge(Some(notification)) = Hoge::new(notification)
                .convert_and_then::<Cancel>(handle_cancel_notification)
                .convert_and_then::<DidChangeTextDocument>(
                    handle_did_change_text_document_notification,
                )
                .convert_and_then::<DidOpenTextDocument>(handle_did_open_text_document_notification)
                .convert_and_then::<DidSaveTextDocument>(handle_did_save_text_document_notification)
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
fn handle_cancel_notification(cancel: <Cancel as LspTypesNotification>::Params) {
    warn!(
        "request {:?} was cenceled, but currently cancellation is not implemented",
        cancel.id
    )
}

fn handle_did_open_text_document_notification(
    did_open_text_document: <DidOpenTextDocument as LspTypesNotification>::Params,
) {
    let src = did_open_text_document.text_document.text;
    let (errors, warnings) = compile::compile_src(src);
    for error in errors {
        warn!("parse error: {}", error);
    }
    for warning in warnings {
        warn!("parse warning: {}", warning);
    }
}

fn handle_did_change_text_document_notification(
    did_change_text_document: <DidChangeTextDocument as LspTypesNotification>::Params,
) {
    let src = did_change_text_document
        .content_changes
        .into_iter()
        .nth(0)
        .unwrap()
        .text;
    let (errors, warnings) = compile::compile_src(src);
    for error in errors {
        warn!("parse error: {}", error);
    }
    for warning in warnings {
        warn!("parse warning: {}", warning);
    }
}

fn handle_did_save_text_document_notification(
    _did_save_text_document: <DidSaveTextDocument as LspTypesNotification>::Params,
) {
    // do nothing
}

struct Hoge(Option<lsp_server::Notification>);
impl Hoge {
    fn new(notification: lsp_server::Notification) -> Self {
        Self(Some(notification))
    }
    fn convert_and_then<T: lsp_types::notification::Notification>(
        self,
        action: fn(T::Params),
    ) -> Self {
        if let Some(notification) = self.0 {
            match notification.extract(T::METHOD) {
                Ok(n) => {
                    action(n);
                    Hoge(None)
                }
                Err(notification) => Hoge(Some(notification)),
            }
        } else {
            Hoge(None)
        }
    }
}

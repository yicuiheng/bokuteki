use log::info;
use lsp_server::{Connection, Message};
use lsp_types::{InitializeParams, ServerCapabilities};
use std::sync::Arc;

pub async fn run() {
    let (connection, io_threads) = Connection::stdio();
    let connection = Arc::new(connection);
    let mut capabilities = ServerCapabilities::default();
    // TODO: change capabilities
    let initialize_params: InitializeParams = serde_json::from_value(
        connection
            .initialize(serde_json::to_value(&mut capabilities).unwrap())
            .unwrap(),
    )
    .unwrap();

    let root_dir = initialize_params.root_uri.unwrap().to_file_path().unwrap();

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
        tokio::task::spawn(handle_msg(msg));
    }
    io_threads.join().unwrap();
}

async fn handle_msg(msg: Message) {
    match msg {
        Message::Request(_) => todo!(),
        Message::Notification(_) => todo!(),
        Message::Response(_) => todo!(),
    }
}

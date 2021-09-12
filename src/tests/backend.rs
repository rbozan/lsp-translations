use super::Backend;

use std::iter::Iterator;

#[cfg(test)]
mod tests {
    use tower_lsp::{lsp_types::*, MessageStream};
    use tower_test::mock::Spawn;

    use super::*;
    use tower_lsp::jsonrpc::{Incoming, Outgoing};
    use tower_lsp::jsonrpc::{Response, Result};
    use tower_lsp::{Client, LanguageServer, LspService, Server};

    use core::task::Poll;

    use futures::join;
    use futures::{future, FutureExt, StreamExt};

    // use core::stream::Stream;

    const INITIALIZE_REQUEST: &str =
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#;
    const INITIALIZED_REQUEST: &str = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;

    const INITIALIZED_NOTIF: &str = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
    const SHUTDOWN_REQUEST: &str = r#"{"jsonrpc":"2.0","method":"shutdown","id":1}"#;
    const EXIT_NOTIF: &str = r#"{"jsonrpc":"2.0","method":"exit"}"#;

    const INITIALIZE_RESPONSE: &str = r#"{
            "jsonrpc":"2.0",
            "result": {
                "capabilities": {
                    "completionProvider": {"resolveProvider": true},
                    "hoverProvider": true,
                    "textDocumentSync": 1,
                    "workspace": {"workspaceFolders": {"changeNotifications": true, "supported": true}}
                }
            },
            "id":1
        }"#;

    fn init_service() -> (Spawn<LspService>, MessageStream) {
        let (service, messages) = LspService::new(|client| Backend::new(client));
        (Spawn::new(service), messages)
    }

    #[tokio::test]
    #[timeout(5000)]
    async fn initialize() {
        let (mut service, _) = init_service();

        let initialize: Incoming = serde_json::from_str(INITIALIZE_REQUEST).unwrap();
        let ok = Outgoing::Response(serde_json::from_str::<Response>(INITIALIZE_RESPONSE).unwrap());
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize.clone()).await, Ok(Some(ok)));

        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let err = Outgoing::Response(serde_json::from_str::<Response>(raw).unwrap());
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize).await, Ok(Some(err)));
    }

    #[tokio::test]
    #[timeout(5000)]
    async fn send_configuration() {
        let (mut service, mut messages) = init_service();

        let initialize: Incoming = serde_json::from_str(INITIALIZE_REQUEST).unwrap();
        let ok = Outgoing::Response(serde_json::from_str::<Response>(INITIALIZE_RESPONSE).unwrap());
        assert_eq!(service.call(initialize.clone()).await, Ok(Some(ok)));

        let shutdown: Incoming = serde_json::from_str(INITIALIZED_REQUEST).unwrap();
        let raw = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        println!("done3");
        // println!("message1 {:?}", messages);
        //

        join!(service.call(shutdown.clone()), testmethod(messages));
        println!("test");

        assert_eq!(service.call(shutdown.clone()).await, Ok(None));

        println!("done4");
    }

    async fn testmethod(mut messages: MessageStream) -> String {
        let message = messages.next().await;
        println!("msg {:?}", message);
        return "finished".to_string();
    }
    /*
    #[tokio::test]
    async fn exit_notification() {
        let (service, _) = LspService::new(|_| Mock::default());
        let mut service = Spawn::new(service);

        let initialized: Incoming = serde_json::from_str(INITIALIZED_NOTIF).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialized.clone()).await, Ok(None));

        let exit: Incoming = serde_json::from_str(EXIT_NOTIF).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(exit).await, Ok(None));

        assert_eq!(service.poll_ready(), Poll::Ready(Err(ExitedError)));
        assert_eq!(service.call(initialized).await, Err(ExitedError));
    } */
}

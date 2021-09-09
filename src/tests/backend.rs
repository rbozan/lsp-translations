use serde_json::Value;
use tokio::net::TcpListener;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};


#[cfg(test)]
mod foo {
    #[tokio::test]
    async fn initializes_only_once() {
        let (service, _) = LspService::new(|_| Mock::default());
        let mut service = Spawn::new(service);

        let initialize: Incoming = serde_json::from_str(INITIALIZE_REQUEST).unwrap();
        let raw = r#"{"jsonrpc":"2.0","result":{"capabilities":{}},"id":1}"#;
        let ok = serde_json::from_str(raw).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize.clone()).await, Ok(Some(ok)));

        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let err = serde_json::from_str(raw).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize).await, Ok(Some(err)));
    }

    #[tokio::test]
    async fn refuses_requests_after_shutdown() {
        let (service, _) = LspService::new(|_| Mock::default());
        let mut service = Spawn::new(service);

        let initialize: Incoming = serde_json::from_str(INITIALIZE_REQUEST).unwrap();
        let raw = r#"{"jsonrpc":"2.0","result":{"capabilities":{}},"id":1}"#;
        let ok = serde_json::from_str(raw).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize.clone()).await, Ok(Some(ok)));

        let shutdown: Incoming = serde_json::from_str(SHUTDOWN_REQUEST).unwrap();
        let raw = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;
        let ok = serde_json::from_str(raw).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(shutdown.clone()).await, Ok(Some(ok)));

        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let err = serde_json::from_str(raw).unwrap();
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(shutdown).await, Ok(Some(err)));
    }

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
    }
}

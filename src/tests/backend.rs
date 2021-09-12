use super::Backend;

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::*;
    use tower_test::mock::Spawn;

    use super::*;
    use tower_lsp::jsonrpc::{Incoming, Outgoing};
    use tower_lsp::jsonrpc::{Response, Result};
    use tower_lsp::{Client, LanguageServer, LspService, Server};

    const INITIALIZE_REQUEST: &str =
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#;
    const INITIALIZED_NOTIF: &str = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
    const SHUTDOWN_REQUEST: &str = r#"{"jsonrpc":"2.0","method":"shutdown","id":1}"#;
    const EXIT_NOTIF: &str = r#"{"jsonrpc":"2.0","method":"exit"}"#;

    fn init_service() -> Spawn<LspService> {
        let (service, _) = LspService::new(|client| Backend::new(client));
        Spawn::new(service)
    }

    #[tokio::test]
    async fn initialize() {
        let mut service = init_service();

        let initialize: Incoming = serde_json::from_str(INITIALIZE_REQUEST).unwrap();
        let raw = r#"{"jsonrpc":"2.0","result":{"capabilities":{}},"id":1}"#;
        // let ok = serde_json::from_str(raw).unwrap();
        let ok = Outgoing::Response(serde_json::from_str::<Response>(raw).unwrap());
        // assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize.clone()).await, Ok(Some(ok)));

        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let err = Outgoing::Response(serde_json::from_str::<Response>(raw).unwrap());
        // assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(initialize).await, Ok(Some(err)));
    }
    /*
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
    } */
}

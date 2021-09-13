use super::Backend;

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tower_lsp::{lsp_types::*, MessageStream};
    use tower_test::mock::Spawn;

    use super::*;
    use tower_lsp::jsonrpc::{ClientRequest, Incoming, Outgoing};
    use tower_lsp::jsonrpc::{Response, Result};
    use tower_lsp::{Client, LanguageServer, LspService, Server};

    use core::task::Poll;

    use futures::join;
    use futures::{future, FutureExt, StreamExt};

    // use core::stream::Stream;

    const INITIALIZED_REQUEST: &str = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;

    lazy_static! {
        static ref INITIALIZE_REQUEST: Incoming = serde_json::from_str(
            r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#
        )
        .unwrap();
        static ref WORKSPACE_CONFIGURATION_REQUEST: Incoming = serde_json::from_str(
            r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": [
            "./backend/cache/translations.json"
        ],
        "fileName": {
            "details": "testdsddasdasdddsad"
        },
        "key": {
            "details": ".",
            "filter": ""
        },
        "trace": {
            "server": "verbose"
        }
    }
], "id": 0 }"#
        )
        .unwrap();
    }

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
    #[timeout(2000)]
    async fn initialize() {
        let (mut service, _) = init_service();

        let ok = Outgoing::Response(serde_json::from_str::<Response>(INITIALIZE_RESPONSE).unwrap());
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(service.call(INITIALIZE_REQUEST.clone()).await, Ok(Some(ok)));

        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let err = Outgoing::Response(serde_json::from_str::<Response>(raw).unwrap());
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(
            service.call(INITIALIZE_REQUEST.clone()).await,
            Ok(Some(err))
        );
    }

    #[tokio::test]
    #[timeout(2000)]
    async fn send_configuration() {
        let (mut service, mut messages) = init_service();

        let ok = Outgoing::Response(serde_json::from_str::<Response>(INITIALIZE_RESPONSE).unwrap());
        assert_eq!(service.call(INITIALIZE_REQUEST.clone()).await, Ok(Some(ok)));

        let shutdown: Incoming = serde_json::from_str(INITIALIZED_REQUEST).unwrap();

        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        //
        join!(
            service.call(shutdown.clone()),
            handle_lsp_message(
                service,
                &mut messages,
                vec![&*WORKSPACE_CONFIGURATION_REQUEST]
            ),
        );
        println!("test we xijn hier!!!!");

        println!("done4");
    }

    async fn handle_lsp_message(
        mut service: Spawn<LspService>,
        messages: &mut MessageStream,
        responses: Vec<&Incoming>,
    ) -> String {
        while let Some(message) = messages.next().await {
            println!("msg {:?}", message);

            match message {
                Outgoing::Response(_) => todo!(),
                Outgoing::Request(req) => {
                    let value = serde_json::to_value(req).unwrap();
                    if value["method"] == "window/logMessage" {
                        println!("Received a window/logMessage, ignoring.");
                        continue;
                    }
                }
            }

            let result = service.call(responses[0].clone()).await;
            println!("result of call {:?}", result);
        }
        return "finished".to_string();
    }
}

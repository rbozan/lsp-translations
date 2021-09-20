#[cfg(test)]
mod tests {

    use tower_lsp::MessageStream;
    use tower_test::mock::Spawn;

    use super::*;
    use tower_lsp::jsonrpc::Response;
    use tower_lsp::jsonrpc::{Incoming, Outgoing};
    use tower_lsp::{Client, LanguageServer, LspService, Server};

    use core::task::Poll;

    use std::env;

    use crate::Backend;

    use futures::select;
    use futures::{FutureExt, StreamExt};

    lazy_static! {
        static ref INITIALIZE_REQUEST: Incoming = serde_json::from_str(
            r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#
        )
        .unwrap();


        static ref INITIALIZE_RESPONSE: Outgoing = Outgoing::Response(serde_json::from_str(r#"{
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
        }"#).unwrap());

        static ref INITIALIZED_REQUEST: Incoming = serde_json::from_str(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#).unwrap();

        static ref WORKSPACE_CONFIGURATION_REQUEST: Incoming = serde_json::from_str(
            r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": [
            "./fixtures/*.json"
        ],
        "fileName": {
            "details": "testdsddasdasdddsad"
        },
        "key": {
            "details": "^.+?\\.(?P<language>.+?)\\.",
            "filter": "^.+?\\..+?\\.(.+$)"
        },
        "trace": {
            "server": "verbose"
        }
    }
], "id": 0 }"#
        )
        .unwrap();

        static ref WORKSPACE_WORKSPACE_FOLDERS_REQUEST: Incoming = serde_json::from_str(
            format!(
                r#"
            {{
                "jsonrpc": "2.0",
                "id": 1,
                "result": [
                    {{
                        "uri": "file://{:}",
                        "name": "recharge-mobile-app"
                    }}
                ]
            }}"#,
                env::current_dir()
                    .unwrap()
                    .join("src")
                    .join("tests")
                    .to_str()
                    .unwrap()
                    .escape_default()
                    .to_string()
            )
            .as_str()
        )
        .unwrap();

        static ref COMPLETION_REQUEST: Incoming = serde_json::from_str(r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/completion",
            "params":{
                "textDocument": {
                    "uri": "file:///home/rbozan/Projects/recharge-mobile-app/backend/src/route/v1/translation.js"
                },
                "position": {
                    "line": 5,
                    "character": 0
                },
                "context": {
                    "triggerKind": 1
                }
            },
            "id":1
        }"#).unwrap();

        static ref COMPLETION_RESPONSE: Outgoing =  Outgoing::Response(serde_json::from_str(r#"{
            "jsonrpc": "2.0",
            "result": [
            {
              "detail": "definition: Definition {\n    key: \"1234567890.en-us.main.content.heading.body\",\n    cleaned_key: Some(\n        \"main.content.heading.body\",\n    ),\n    file: \"\",\n    language: None,\n    value: \"This is the body of my website.\",\n}",
              "kind": 1,
              "label": "main.content.heading.body"
            },
            {
              "detail": "definition: Definition {\n    key: \"1234567890.en-us.main.content.heading.title\",\n    cleaned_key: Some(\n        \"main.content.heading.title\",\n    ),\n    file: \"\",\n    language: None,\n    value: \"A regular header for my content\",\n}",
              "kind": 1,
              "label": "main.content.heading.title"
            },
            {
              "detail": "definition: Definition {\n    key: \"1234567890.en-us.main.header.title\",\n    cleaned_key: Some(\n        \"main.header.title\",\n    ),\n    file: \"\",\n    language: None,\n    value: \"This title will appear in the header.\",\n}",
              "kind": 1,
              "label": "main.header.title"
            }
            ],
            "id": 1
        }"#).unwrap());
    }

    fn init_service() -> (Spawn<LspService>, MessageStream) {
        let (service, messages) = LspService::new(|client| Backend::new(client));
        (Spawn::new(service), messages)
    }

    #[tokio::test]
    #[timeout(2000)]
    async fn initialize() {
        let (mut service, _) = init_service();

        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(
            service.call(INITIALIZE_REQUEST.clone()).await,
            Ok(Some(INITIALIZE_RESPONSE.clone()))
        );

        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let err = Outgoing::Response(serde_json::from_str::<Response>(raw).unwrap());
        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));
        assert_eq!(
            service.call(INITIALIZE_REQUEST.clone()).await,
            Ok(Some(err))
        );
    }

    #[tokio::test]
    #[timeout(8000)]
    async fn send_configuration() {
        let (mut service, mut messages) = init_service();

        assert_eq!(
            service.call(INITIALIZE_REQUEST.clone()).await,
            Ok(Some(INITIALIZE_RESPONSE.clone()))
        );

        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));

        select!(
            req = service.call(INITIALIZED_REQUEST.clone()).fuse() => {
                assert_eq!(req.unwrap(), None);
            },
            () = handle_lsp_message(
                &mut service,
                &mut messages,
                vec![
                    &*WORKSPACE_CONFIGURATION_REQUEST,
                    &*WORKSPACE_WORKSPACE_FOLDERS_REQUEST,
                ],
            ).fuse() => {
                panic!("lsp messages should not finish faster than request")
            },
        );

        let message = (messages.next().await).unwrap();
        let value = serde_json::to_value(message).unwrap();

        assert_eq!(value["params"]["message"], "Loaded 3 definitions");
    }

    #[tokio::test]
    #[timeout(8000)]
    async fn completion() {
        let (mut service, _) = prepare_workspace().await;

        // println!("{:}", serde_json::to_string_pretty(&(&service.call(COMPLETION_REQUEST.clone()).await).as_ref().unwrap()).unwrap());

        assert_eq!(
            service.call(COMPLETION_REQUEST.clone()).await,
            Ok(Some(COMPLETION_RESPONSE.clone()))
        );
    }

    // Helper function
    async fn handle_lsp_message(
        service: &mut Spawn<LspService>,
        messages: &mut MessageStream,
        responses: Vec<&Incoming>,
    ) {
        let mut i = 0;
        while let Some(message) = messages.next().await {
            match message {
                Outgoing::Response(_) => todo!(),
                Outgoing::Request(req) => {
                    let value = serde_json::to_value(req.clone()).unwrap();
                    if value["method"] == "window/logMessage" {
                        println!(
                            "[window/logMessage] {:?}",
                            value["params"]["message"].as_str().unwrap()
                        );
                    } else {
                        println!("[msg request] {:?}", &req);

                        let result = service.call(responses[i].clone()).await;
                        println!("[msg response] {:?}", result);
                        i = i + 1;
                    }
                }
            }

            // if i >= responses.len() { break; }
        }
    }

    async fn prepare_workspace() -> (Spawn<LspService>, MessageStream) {
        let (mut service, mut messages) = init_service();

        assert_eq!(
            service.call(INITIALIZE_REQUEST.clone()).await,
            Ok(Some(INITIALIZE_RESPONSE.clone()))
        );

        assert_eq!(service.poll_ready(), Poll::Ready(Ok(())));

        select!(
            req = service.call(INITIALIZED_REQUEST.clone()).fuse() => {
                assert_eq!(req.unwrap(), None);
            },
            () = handle_lsp_message(
                &mut service,
                &mut messages,
                vec![
                    &*WORKSPACE_CONFIGURATION_REQUEST,
                    &*WORKSPACE_WORKSPACE_FOLDERS_REQUEST,
                ],
            ).fuse() => {
                panic!("lsp messages should not finish faster than finishing request")
            },
        );

        let message = (messages.next().await).unwrap();
        let value = serde_json::to_value(message).unwrap();

        (Spawn::new(service.into_inner()), messages)
    }
}

use tower_lsp::jsonrpc::Response;
use tower_lsp::jsonrpc::{Incoming, Outgoing};

use core::task::Poll;

use std::env;

mod helpers;
use helpers::*;

lazy_static! {
    static ref INITIALIZE_REQUEST: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#
    )
    .unwrap();


    static ref INITIALIZE_RESPONSE: Outgoing = Outgoing::Response(serde_json::from_str(r#"{
            "jsonrpc":"2.0",
            "result": {
                "capabilities": {
                    "completionProvider": {
                        "resolveProvider": true,
                        "triggerCharacters": ["'", "\"", "`", "."]
                    },
                    "hoverProvider": true,
                    "textDocumentSync": 2,
                    "workspace": {"workspaceFolders": {"changeNotifications": true, "supported": true}}
                }
            },
            "id":1
        }"#).unwrap());

    static ref INITIALIZED_REQUEST: Incoming = serde_json::from_str(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#).unwrap();

    static ref WORKSPACE_CONFIGURATION_REQUEST: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": {
            "include": [
                "./fixtures/*.json",
                ".\\fixtures\\*.json"
            ]
        },
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
        )
        .as_str()
    )
    .unwrap();
}

#[tokio::test]
#[timeout(500)]
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
#[allow(unused_must_use)]
#[timeout(500)]
async fn send_configuration() {
    prepare_workspace().await;
}

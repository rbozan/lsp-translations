use tower_lsp::jsonrpc::{Incoming, Outgoing};

use std::env;

mod helpers;
use helpers::*;

// use helpers;

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

    static ref COMPLETION_RESPONSE: Outgoing =  Outgoing::Response(serde_json::from_str(r#"
{
  "jsonrpc": "2.0",
  "result": [
    {
      "kind": 1,
      "label": "main.content.heading.body"
    },
    {
      "kind": 1,
      "label": "main.content.heading.title"
    },
    {
      "kind": 1,
      "label": "main.header.title"
    }
  ],
  "id": 1
}
"#).unwrap());

    static ref COMPLETION_RESOLVE_REQUEST: Incoming = serde_json::from_str(r#"{
            "jsonrpc": "2.0",
            "method": "completionItem/resolve",
            "params": {
                "label": "main.header.title",
                "insertTextFormat": 1,
                "kind": 1
            },
            "id": 1
        }"#).unwrap();
}

#[tokio::test]
#[timeout(500)]
async fn completion() {
    let (mut service, _) = prepare_workspace().await;

    // did open text document

    assert_eq!(
        service.call(COMPLETION_REQUEST.clone()).await,
        Ok(Some(COMPLETION_RESPONSE.clone()))
    );
}

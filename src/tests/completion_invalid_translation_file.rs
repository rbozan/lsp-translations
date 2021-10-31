use tower_lsp::jsonrpc::{Incoming, Outgoing};

mod helpers;
use helpers::*;

#[cfg(test)]
use pretty_assertions::assert_eq;

lazy_static! {
    static ref WORKSPACE_CONFIGURATION_REQUEST: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": {
            "include": [
                "./fixtures/invalid_translation_files/*.yml"
            ]
        },
        "fileName": {
            "details": ""
        },
        "key": {
        },
        "trace": {
            "server": "verbose"
        }
    }
], "id": 1 }"#
    )
    .unwrap();
    static ref DID_OPEN_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/didOpen",
            "params":{
                "textDocument": {
                    "uri": "file:///somefile.js",
                    "languageId": "javascript",
                    "version": 1,
                    "text": "translate('')"
                }
            },
            "id": 2
        }"#
    )
    .unwrap();
    static ref COMPLETION_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/completion",
            "params":{
                "textDocument": {
                    "uri": "file:///somefile.js"
                },
                "position": {
                    "line": 0,
                    "character": 11
                },
                "context": {
                    "triggerKind": 1
                }
            },
            "id": 2
        }"#
    )
    .unwrap();
    static ref COMPLETION_RESPONSE: Outgoing = Outgoing::Response(
        serde_json::from_str(
            r#"
{
   "jsonrpc":"2.0",
   "result":[],
   "id": 2
}
"#
        )
        .unwrap()
    );
}

#[tokio::test]
#[timeout(500)]
async fn completion() {
    let (mut service, _) = prepare_with_workspace_config(&WORKSPACE_CONFIGURATION_REQUEST).await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));

    assert_eq!(
        service.call(COMPLETION_REQUEST.clone()).await,
        Ok(Some(COMPLETION_RESPONSE.clone()))
    );
}

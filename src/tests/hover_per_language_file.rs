use tower_lsp::jsonrpc::{Incoming, Outgoing, Response};

mod helpers;
use helpers::*;

// use helpers;

lazy_static! {
    static ref DID_OPEN_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/didOpen",
            "params":{
                "textDocument": {
                    "uri": "file:///somefile.js",
                    "languageId": "javascript",
                    "version": 1,
                    "text": "translate('test');"
                }
            },
            "id":1
        }"#
    )
    .unwrap();
    static ref WORKSPACE_CONFIGURATION_REQUEST_WITH_LANGUAGE: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": {
            "include": [
                "./fixtures/per_language_file/*.json"
            ]
        },
        "fileName": {
            "details": "(?P<language>.+?)\\."
        },
        "key": {
            "filter": ""
        },
        "trace": {
            "server": "verbose"
        }
    }
], "id": 1 }"#
    )
    .unwrap();
    static ref HOVER_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///somefile.js"
                },
                "position": {
                    "line": 0,
                    "character": 11
                }
            },
            "id":1
        }"#
    )
    .unwrap();
    static ref HOVER_RESPONSE_WITH_LANGUAGE: Outgoing = Outgoing::Response(
        serde_json::from_str(
            r#"
{
   "jsonrpc":"2.0",
   "result":{
      "contents":"flag|language|translation\n-|-|-\n🇳🇱|**nl**|Nederlands\n🇺🇸|**en**|English",
      "range":{
         "end":{
            "character": 15,
            "line":0
         },
         "start":{
            "character": 11,
            "line":0
         }
      }
   },
   "id":1
}
"#
        )
        .unwrap()
    );
    static ref INITIALIZE_REQUEST: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#
    )
    .unwrap();
    static ref WORKSPACE_CONFIGURATION_REQUEST_WITHOUT_LANGUAGE: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": {
            "include": [
                "./fixtures/per_language_file/*.json"
            ]
        },
        "fileName": {
            "details": ""
        },
        "key": {
            "filter": ""
        },
        "trace": {
            "server": "verbose"
        }
    }
], "id": 1 }"#
    )
    .unwrap();
    static ref HOVER_RESPONSE_WITHOUT_LANGUAGE: Outgoing = Outgoing::Response(
        serde_json::from_str(
            r#"
{
   "jsonrpc":"2.0",
   "result":{
      "contents": "|translation|\n|-\n|Nederlands\n|English",
      "range":{
         "end":{
            "character": 15,
            "line":0
         },
         "start":{
            "character": 11,
            "line":0
         }
      }
   },
   "id":1
}
"#
        )
        .unwrap()
    );
}

#[tokio::test]
#[timeout(500)]
async fn hover() {
    let (mut service, _) =
        prepare_with_workspace_config(&WORKSPACE_CONFIGURATION_REQUEST_WITH_LANGUAGE).await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));

    assert_eq!(
        service.call(HOVER_REQUEST.clone()).await,
        Ok(Some(HOVER_RESPONSE_WITH_LANGUAGE.clone()))
    );
}

#[tokio::test]
#[timeout(500)]
async fn hover_without_language() {
    let (mut service, _) =
        prepare_with_workspace_config(&WORKSPACE_CONFIGURATION_REQUEST_WITHOUT_LANGUAGE).await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));

    assert_eq!(
        service.call(HOVER_REQUEST.clone()).await,
        Ok(Some(HOVER_RESPONSE_WITHOUT_LANGUAGE.clone()))
    );
}

use tower_lsp::jsonrpc::{Incoming, Outgoing};

mod helpers;
use helpers::*;

// use helpers;

lazy_static! {
    static ref DID_OPEN_REQUEST: Incoming = serde_json::from_str(r#"{
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
            "id":1
        }"#).unwrap();

    static ref COMPLETION_REQUEST: Incoming = serde_json::from_str(r#"{
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
            "id":1
        }"#).unwrap();

    static ref COMPLETION_RESPONSE: Outgoing =  Outgoing::Response(serde_json::from_str(r#"
{
  "jsonrpc": "2.0",
  "result": [
    {
      "kind": 1,
      "label": "main.header.title",
      "textEdit": {
          "newText": "main.header.title",
          "range": {
              "start": { "character": 11, "line": 0 },
              "end": { "character": 11, "line": 0 }
          }
      }
    },
    {
      "kind": 1,
      "label": "main.content.heading.title",
      "textEdit": {
          "newText": "main.content.heading.title",
          "range": {
              "start": { "character": 11, "line": 0 },
              "end": { "character": 11, "line": 0 }
          }
      }
    },
    {
      "kind": 1,
      "label": "main.content.heading.body",
      "textEdit": {
          "newText": "main.content.heading.body",
          "range": {
              "start": { "character": 11, "line": 0 },
              "end": { "character": 11, "line": 0 }
          }
      }

    }
  ],
  "id": 1
}
"#).unwrap());

    // TODO: Add this test
    /* static ref COMPLETION_RESOLVE_REQUEST: Incoming = serde_json::from_str(r#"{
            "jsonrpc": "2.0",
            "method": "completionItem/resolve",
            "params": {
                "label": "main.header.title",
                "insertTextFormat": 1,
                "kind": 1
            },
            "id": 1
        }"#).unwrap(); */
}

#[tokio::test]
#[timeout(500)]
async fn completion() {
    let (mut service, _) = prepare_workspace().await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));

    assert_eq!(
        service.call(COMPLETION_REQUEST.clone()).await,
        Ok(Some(COMPLETION_RESPONSE.clone()))
    );
}

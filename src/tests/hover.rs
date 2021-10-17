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
                    "text": "translate('main.header.title'); translate('some.unknown.translation')"
                }
            },
            "id":1
        }"#
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

    static ref HOVER_RESPONSE: Outgoing = Outgoing::Response(
        serde_json::from_str(
            r#"
{
   "jsonrpc":"2.0",
   "result":{
      "contents":"flag|language|translation\n-|-|-\n🇺🇸|**en-us**|This title will appear in the header.",
      "range":{
         "end":{
            "character":28,
            "line":0
         },
         "start":{
            "character":11,
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


        static ref HOVER_ON_UNKNOWN_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///somefile.js"
                },
                "position": {
                    "line": 0,
                    "character": 45
                }
            },
            "id":1
        }"#
    )
    .unwrap();

}

#[tokio::test]
#[timeout(500)]
async fn hover() {
    let (mut service, _) = prepare_workspace().await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));

    assert_eq!(
        service.call(HOVER_REQUEST.clone()).await,
        Ok(Some(HOVER_RESPONSE.clone()))
    );
}

#[tokio::test]
#[timeout(500)]
async fn hover_on_unknown_key_returns_nothing() {
    let (mut service, _) = prepare_workspace().await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));

    assert_eq!(
        service.call(HOVER_ON_UNKNOWN_REQUEST.clone()).await,
        Ok(Some(Outgoing::Response(Response::ok(
            tower_lsp::jsonrpc::Id::Number(1),
            serde_json::Value::default()
        ))))
    );
}

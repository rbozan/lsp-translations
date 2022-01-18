use tower_lsp::jsonrpc::Incoming;

mod helpers;
use helpers::*;

#[cfg(test)]
use pretty_assertions::assert_eq;

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
                    "text": ""
                }
            },
            "id": 2
        }"#
    )
    .unwrap();
    static ref DID_CHANGE_ADD_TEXT_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/didChange",
            "params":{
                "textDocument": {
                    "uri": "file:///somefile.js",
                    "version": 2
                },
                "contentChanges": [
                    {
                        "range": {
                            "start": {
                                "line": 0,
                                "character": 0
                            },
                            "end": {
                                "line": 0,
                                "character": 0
                            }
                        },
                        "rangeLength": 0,
                        "text": "test🇳🇱 "
                    }
                ]
            },
            "id": 2
        }"#
    )
    .unwrap();
    static ref DID_CHANGE_REMOVE_TEXT_REQUEST: Incoming = serde_json::from_str(
        r#"{
            "jsonrpc":"2.0",
            "method":"textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///somefile.js",
                    "version": 3
                },
                "contentChanges": [
                    {
                        "range": {
                            "start": {
                                "line": 0,
                                "character": 0
                            },
                            "end": {
                                "line": 0,
                                "character": 9
                            }
                        },
                        "rangeLength": 9,
                        "text": ""
                    }
                ]
            },
            "id": 3
        }"#
    )
    .unwrap();
}

#[tokio::test]
#[timeout(500)]
async fn change_emoji_in_document() {
    let (mut service, _) = prepare_workspace().await;

    assert_eq!(service.call(DID_OPEN_REQUEST.clone()).await, Ok(None));
    assert_eq!(
        service.call(DID_CHANGE_ADD_TEXT_REQUEST.clone()).await,
        Ok(None)
    );
    assert_eq!(
        service.call(DID_CHANGE_REMOVE_TEXT_REQUEST.clone()).await,
        Ok(None)
    );
}

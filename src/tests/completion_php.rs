use tower_lsp::{
    jsonrpc::{Incoming, Outgoing, Response},
    lsp_types::{CompletionItem, CompletionItemKind, CompletionTextEdit, Position, TextEdit},
};

mod helpers;
use helpers::*;

#[cfg(test)]
use pretty_assertions::assert_eq;

// use helpers;

lazy_static! {
    static ref WORKSPACE_CONFIGURATION_REQUEST: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": {
            "include": [
                "./fixtures/*.php"
            ]
        },
        "fileName": {
            "details": ""
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
                    "uri": "file:///somefile.php",
                    "languageId": "php",
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
                    "uri": "file:///somefile.php"
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
    static ref COMPLETION_RESPONSE: Outgoing = {
        let keys = [
            "test-single",
            "test-multiline",
        ];

        let completion_items = keys
            .iter()
            .map(|key| CompletionItem {
                label: key.to_string(),
                kind: Some(CompletionItemKind::Text),
                detail: None,
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: tower_lsp::lsp_types::Range::new(
                        Position {
                            line: 0,
                            character: 11,
                        },
                        Position {
                            line: 0,
                            character: 11,
                        },
                    ),
                    new_text: key.to_string(),
                })),
                ..Default::default()
            })
            .collect::<Vec<CompletionItem>>();

        Outgoing::Response(Response::ok(
            tower_lsp::jsonrpc::Id::Number(2),
            serde_json::to_value(completion_items).unwrap(),
        ))
    };
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

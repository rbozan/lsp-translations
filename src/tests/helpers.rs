use tower_lsp::MessageStream;
use tower_test::mock::Spawn;

use tower_lsp::jsonrpc::{Incoming, Outgoing};
use tower_lsp::LspService;

use core::task::Poll;

use crate::Backend;

use futures::select;
use futures::{FutureExt, StreamExt};
use std::env;

#[cfg(test)]
use pretty_assertions::assert_eq;

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
], "id": 1 }"#
    )
    .unwrap();

    static ref WORKSPACE_WORKSPACE_FOLDERS_REQUEST: Incoming = serde_json::from_str(
        format!(
            r#"
            {{
                "jsonrpc": "2.0",
                "id": 2,
                "result": [
                    {{
                        "uri": "file://{:}",
                        "name": "test-project"
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


    static ref MESSAGE_OK_RESPONSE_CONFIG_CAPABILITY: Incoming = serde_json::from_str(
        r#"
        {
            "jsonrpc": "2.0",
            "id": 0,
            "result": null
        }"#
    )
    .unwrap();


    static ref MESSAGE_OK_RESPONSE_WATCH_CAPABILITY: Incoming = serde_json::from_str(
        r#"
        {
            "jsonrpc": "2.0",
            "id": 3,
            "result": null
        }"#
    )
    .unwrap();

}

pub fn init_service() -> (Spawn<LspService>, MessageStream) {
    let (service, messages) = LspService::new(Backend::new);
    (Spawn::new(service), messages)
}

pub async fn handle_lsp_message(
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
                    println!("[msg response #{}] {:?}", i, result);
                    i += 1;
                }
            }
        }
    }
}

pub async fn prepare_workspace() -> (Spawn<LspService>, MessageStream) {
    prepare_with_workspace_config(&WORKSPACE_CONFIGURATION_REQUEST).await
}

pub async fn prepare_with_workspace_config(
    workspace_config_request: &Incoming,
) -> (Spawn<LspService>, MessageStream) {
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
                &MESSAGE_OK_RESPONSE_CONFIG_CAPABILITY,
                workspace_config_request,
                &WORKSPACE_WORKSPACE_FOLDERS_REQUEST,
                &MESSAGE_OK_RESPONSE_WATCH_CAPABILITY
            ],
        ).fuse() => {
            panic!("lsp messages should not finish faster than finishing request")
        },
    );

    (Spawn::new(service.into_inner()), messages)
}

use tower_lsp::jsonrpc::{Incoming, Outgoing};

mod helpers;
use helpers::*;

#[cfg(test)]
use pretty_assertions::{assert_eq};

// use helpers;

lazy_static! {
    static ref WORKSPACE_CONFIGURATION_REQUEST: Incoming = serde_json::from_str(
        r#"{"jsonrpc":"2.0","result": [
    {
        "translationFiles": [
            "./fixtures/*.yml"
        ],
        "fileName": {
            "details": ""
        },
        "key": {
            "details": "^.+?\\.(?P<language>.+?)\\.",
            "filter": "^.+?\\.(.+$)"
        },
        "trace": {
            "server": "verbose"
        }
    }
], "id": 0 }"#
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
            "id":1
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
            "id":1
        }"#
    )
    .unwrap();
    static ref COMPLETION_RESPONSE: Outgoing = Outgoing::Response(
        serde_json::from_str(
            r#"
{
   "jsonrpc":"2.0",
   "result":[
      {
         "kind":1,
         "label":"accounts.edit.new_password",
         "textEdit":{
            "newText":"accounts.edit.new_password",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"accounts.edit.update",
         "textEdit":{
            "newText":"accounts.edit.update",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"employees.assigned_employees.assigned_employee.main_dta",
         "textEdit":{
            "newText":"employees.assigned_employees.assigned_employee.main_dta",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"simple_form.confirm_registration",
         "textEdit":{
            "newText":"simple_form.confirm_registration",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"simple_form.date.abbr_day_names[0]",
         "textEdit":{
            "newText":"simple_form.date.abbr_day_names[0]",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"simple_form.formats.default",
         "textEdit":{
            "newText":"simple_form.formats.default",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"simple_form.new_model",
         "textEdit":{
            "newText":"simple_form.new_model",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"simple_form.no",
         "textEdit":{
            "newText":"simple_form.no",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      },
      {
         "kind":1,
         "label":"simple_form.required.mark",
         "textEdit":{
            "newText":"simple_form.required.mark",
            "range":{
               "start":{
                  "character":11,
                  "line":0
               },
               "end":{
                  "character":11,
                  "line":0
               }
            }
         }
      }
   ],
   "id":1
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

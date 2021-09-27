#[path = "./tests/backend.rs"]
mod tests;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::request::WorkspaceConfiguration;
use lsp_types::request::WorkspaceFoldersRequest;
use lsp_types::CompletionOptions;
use lsp_types::ConfigurationItem;
use lsp_types::ConfigurationParams;
use lsp_types::HoverProviderCapability;
use lsp_types::OneOf;
use lsp_types::WorkspaceFoldersServerCapabilities;
use lsp_types::WorkspaceServerCapabilities;
use lsp_types::{
    request::GotoDefinition, GotoDefinitionResponse, InitializeParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};
use lsp_types::{LogMessageParams, WorkspaceFolder};
// use crossbeam_channel::SendError;
use serde_json::Value;

use glob::glob;
use serde::Deserialize;
use std::cell::Cell;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use itertools::Itertools;

#[macro_use]
extern crate derive_new;

use regex::Regex;

extern crate serde;
extern crate serde_regex;

#[cfg(test)]
#[macro_use]
extern crate ntest;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[derive(Deserialize, Debug, Default)]
struct FileNameConfig {
    #[serde(with = "serde_regex")]
    details: Option<Regex>,
}

#[derive(Deserialize, Debug, Default)]
struct KeyConfig {
    #[serde(with = "serde_regex")]
    details: Option<Regex>,
    #[serde(with = "serde_regex")]
    filter: Option<Regex>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct ExtensionConfig {
    translation_files: Vec<String>,
    file_name: FileNameConfig,
    key: KeyConfig,
}

#[derive(new)]
pub struct Backend {
    connection: Connection,
    initialization_params: Value,
    #[new(value = "Arc::new(Mutex::new(Cell::new(vec![])))")]
    definitions: Arc<Mutex<Cell<Vec<Definition>>>>,
    #[new(value = "Arc::new(Mutex::new(Cell::new(ExtensionConfig::default())))")]
    config: Arc<Mutex<Cell<ExtensionConfig>>>,
}

impl Backend {
    /* impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities         })
    }

    async fn initialized(&self, _: InitializedParams) {
        // Read configuration
        let config = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                // section: Some("lsp-translations.translationFiles".to_string()),
                section: Some("lsp-translations".to_string()),
            }])
            .await; */

    fn request_config(&self) {
        let params: ConfigurationParams = ConfigurationParams {
            items: vec![ConfigurationItem {
                scope_uri: None,
                section: Some("lsp-translations".to_string()),
            }],
        };

        self.connection
            .sender
            .send(Message::Request(Request {
                id: RequestId::from("workspaceConfiguration".to_string()),
                method: "workspace/configuration".to_string(),
                params: serde_json::to_value(params).unwrap(),
            }))
            .unwrap();
    }

    fn read_config(&self, params: Value) {
        // TODO: Move setting config to other function
        let config: ExtensionConfig = serde_json::from_value(params[0].clone()).unwrap();
        self.config.lock().unwrap().set(config);

        self.connection
            .sender
            .send(Message::Request(Request {
                id: RequestId::from("workspaceFolders".to_string()),
                method: "workspace/workspaceFolders".to_string(),
                params: serde_json::Value::default(),
            }))
            .unwrap();
    }

    fn fetch_translations(&self, folders: Vec<WorkspaceFolder>) {
        let files: Vec<PathBuf> = folders
            .iter()
            .map(|folder| {
                self.config
                    .lock()
                    .unwrap()
                    // TODO: Remove mut here
                    .get_mut()
                    .translation_files
                    .iter()
                    .filter_map(|glob_pattern_setting| {
                        match Path::new(&folder.uri.path())
                            .join(glob_pattern_setting)
                            .to_str()
                        {
                            Some(glob_pattern) => match glob(glob_pattern) {
                                Ok(paths) => {
                                    let result: Vec<Option<PathBuf>> = paths
                                        .map(|path| match path {
                                            Ok(path) => {
                                                // panic!("paths glob: glob pattesrn: {:?}", PathBuf::from(&path));
                                                Some(path)
                                            }
                                            Err(_) => None,
                                        })
                                        .collect();

                                    Some(result)
                                }
                                Err(_) => None,
                            },
                            None => None,
                        }
                    })
                    .flatten()
                    .filter_map(|path| path)
                    .collect::<PathBuf>()
            })
            .collect();

        eprintln!("path bufs: {:?}", files);

        self.read_translation(&files[0]).unwrap();

        eprintln!("Loaded {:?} definitions", self.definitions.lock().unwrap().get_mut());
    }

    fn read_translation(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let value: Value = serde_json::from_reader(reader)?;

        let mut definitions = self.definitions.lock().unwrap();
        definitions.get_mut().clear();

        definitions.set(self.parse_translation_structure(&value, "".to_string()));

        Ok(())
    }

    fn parse_translation_structure(&self, value: &Value, json_path: String) -> Vec<Definition> {
        let mut definitions = vec![];

        // println!("parse_translation_structure {:?}", value);

        match value {
            /* Value::Array(values) => values.iter().for_each(|value| {
                definitions.append(&mut self.parse_translation_structure(value));
            }), */
            Value::Object(values) => values.iter().for_each(|(key, value)| {
                definitions.append(&mut self.parse_translation_structure(
                    value,
                    format!(
                        "{}{}{}",
                        json_path,
                        if !json_path.is_empty() { "." } else { "" },
                        key
                    ),
                ));
            }),
            Value::String(value) => {
                let cleaned_key = self
                    .config
                    .lock()
                    .unwrap()
                    .get_mut()
                    .key
                    .filter
                    .as_ref()
                    .and_then(|key_filter_regex| {
                        key_filter_regex.captures(&json_path).and_then(|cap| {
                            println!("cap: {:?}", cap);
                            cap.get(1)
                                .and_then(|group| Some(group.as_str().to_string()))
                        })
                    });

                let language = &self
                    .config
                    .lock()
                    .unwrap()
                    .get_mut()
                    .key
                    .details
                    .as_ref()
                    .and_then(|key_details_regex| {
                        key_details_regex.captures(&json_path).and_then(|cap| {
                            cap.name("language")
                                .and_then(|matches| Some(matches.as_str().to_string()))
                        })
                    });

                // key_filter_regex.captures_iter(&json_path).intersperse(".").collect();
                definitions.push(Definition {
                    key: json_path,
                    cleaned_key,
                    value: value.to_string(),
                    language: language.clone(),
                    ..Default::default()
                })
            }
            _ => println!("TODO: Error, translation file is not an array"),
        }

        definitions
    }

    fn main_loop(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let connection = &self.connection;

        let params: InitializeParams =
            serde_json::from_value(self.initialization_params.clone()).unwrap();
        eprintln!("starting example main loop");
        for msg in &connection.receiver {
            eprintln!("got msg: {:?}", msg);
            match msg {
                Message::Request(req) => {
                    if connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    eprintln!("got request: {:?}", req);
                    if let Ok((id, params)) = cast::<GotoDefinition>(req) {
                        eprintln!("got gotoDefinition request #{}: {:?}", id, params);
                        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    };
                }
                Message::Response(resp) => {
                    eprintln!("got response: {:?}", resp);

                    if resp.id == RequestId::from("workspaceConfiguration".to_string()) {
                        self.read_config(resp.result.unwrap());
                    } else if resp.id == RequestId::from("workspaceFolders".to_string()) {
                        let folders: Vec<WorkspaceFolder> =
                            serde_json::from_value(resp.result.unwrap()).unwrap();
                        self.fetch_translations(folders);
                        // WorkspaceFolder
                    }
                }
                Message::Notification(not) => {
                    eprintln!("got notification: {:?}", not);
                }
            }
        }
        Ok(())
    }
}

/*
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities         })
    }

    async fn initialized(&self, _: InitializedParams) {
        // Read configuration
        let config = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                // section: Some("lsp-translations.translationFiles".to_string()),
                section: Some("lsp-translations".to_string()),
            }])
            .await;

        match config {
            Ok(config) => {
                self.client
                    .log_message(MessageType::Log, format!("config received {:?}", config))
                    .await;

                self.fetch_translations(config[0].clone()).await;
                self.client
                    .log_message(
                        MessageType::Info,
                        format!(
                            "Loaded {:} definitions",
                            self.definitions.lock().unwrap().get_mut().len()
                        ),
                    )
                    .await;
            }
            Err(err) => self.client.log_message(MessageType::Error, err).await,
        }
    }

    async fn completion(&self, _: CompletionParams) -> jsonrpc::Result<Option<CompletionResponse>> {
        if let Ok(ref mut definitions) = self.definitions.try_lock() {
            let definitions = definitions.get_mut();

            Ok(Some(CompletionResponse::Array(
                definitions
                    .iter()
                    .map(|definition| {
                        let detail: String = definitions
                            .iter()
                            .filter(|def| *def == definition)
                            .map(|def| {
                                format!(
                                    "{}{}",
                                    def.language
                                        .as_ref()
                                        .and_then(|lang| Some(lang.to_owned() + ":\n"))
                                        .unwrap_or("".to_string()),
                                    def.value
                                )
                            })
                            .intersperse("\n".to_string())
                            .collect();

                        CompletionItem {
                            label: definition
                                .cleaned_key
                                .as_ref()
                                .unwrap_or(&definition.key)
                                .clone(),
                            kind: Some(CompletionItemKind::Text),
                            detail: Some(detail),
                            // documentation
                            ..Default::default()
                        }
                    })
                    .collect(),
            )))
        } else {
            println!("Gaat fout");
            Err(Error::internal_error())
        }
    }

    async fn hover(&self, _: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        self.client.log_message(MessageType::Info, "hoverrr").await;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "TODO: Hier komt hover informatie over translation".to_string(),
            )),
            range: None,
        }))
    }
}
 */
fn cast<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: None,
            // trigger_characters: Some(vec!["'".to_string(), "\"".to_string()]),
            work_done_progress_options: Default::default(),
            all_commit_characters: None,
        }),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..ServerCapabilities::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities)?;
    let server = Backend::new(connection, initialization_params);
    server.request_config();
    server.main_loop()?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

use merge::Merge;
#[derive(Merge, Default, Debug)]
struct Definition {
    #[merge(skip)]
    key: String,
    cleaned_key: Option<String>,
    #[merge(skip)]
    file: String,
    language: Option<String>,
    #[merge(skip)]
    value: String,
}

impl PartialEq for Definition {
    fn eq(&self, other: &Self) -> bool {
        match &self.cleaned_key {
            Some(_) => self.cleaned_key == other.cleaned_key,
            None => self.key == other.key,
        }
    }
}

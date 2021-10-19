#[path = "./tests/backend.rs"]
#[cfg(test)]
mod tests;

#[path = "./tests/completion.rs"]
#[cfg(test)]
mod tests_completion;

#[path = "./tests/hover.rs"]
#[cfg(test)]
mod tests_hover;

mod lsp_document;
use crate::lsp_document::FullTextDocument;

mod string_helper;
use crate::string_helper::find_translation_key_by_position;
use country_emoji::flag;
use std::convert::TryInto;
use std::path::Path;
use string_helper::get_editing_range;
use string_helper::is_editing_position;
use string_helper::TRANSLATION_BEGIN_CHARS;

use serde_json::Value;
use tower_lsp::jsonrpc::{self, Error};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use glob::glob;
use serde::Deserialize;
use std::cell::Cell;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
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

// #[cfg(test)]
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
    client: Client,
    #[new(value = "Arc::new(Mutex::new(Cell::new(vec![])))")]
    definitions: Arc<Mutex<Cell<Vec<Definition>>>>,
    #[new(value = "Arc::new(Mutex::new(Cell::new(ExtensionConfig::default())))")]
    config: Arc<Mutex<Cell<ExtensionConfig>>>,
    #[new(value = "Arc::new(Mutex::new(Cell::new(vec![])))")]
    documents: Arc<Mutex<Cell<Vec<FullTextDocument>>>>,
}

impl Backend {
    /// Figures out which translation files exists on the system of the user
    /// and calls `read_translation` to append them to the definitions
    async fn fetch_translations(&self, config_value: Value) {
        // TODO: Move setting config to other function
        let config: ExtensionConfig = serde_json::from_value(config_value).unwrap();
        self.config.lock().unwrap().set(config);

        let folders = self.client.workspace_folders().await.unwrap().unwrap();

        self.client
            .log_message(MessageType::Info, format!("folders: {:?}", folders))
            .await;

        // Retrieve the files based on the provided glob patterns
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
                        match &folder
                            .uri
                            .to_file_path()
                            .unwrap()
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
                    .flatten()
                    .collect::<PathBuf>()
            })
            .filter(|path| path.is_file())
            .collect();

        eprintln!("path bufs: {:?}", files);

        // TODO: Unregister capability?

        // Register capability to watch files
        self.client
            .register_capability(vec![Registration {
                id: "workspace/didChangeWatchedFiles".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(
                    serde_json::to_value(
                        files
                            .iter()
                            .map(|file| FileSystemWatcher {
                                glob_pattern: file.to_str().unwrap().to_string(),
                                kind: None,
                            })
                            .collect::<Vec<FileSystemWatcher>>(),
                    )
                    .unwrap(),
                ),
            }])
            .await
            .unwrap();

        // Clear and add definitions
        self.definitions.lock().unwrap().set(vec![]);

        for file in &files {
            (self.read_translation(file)).unwrap();
        }
    }

    /// Reads the translations from a single file and adds them to the `definitions`
    fn read_translation(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let value: Value = serde_json::from_reader(reader)?;

        let mut new_definitions = self.parse_translation_structure(&value, "".to_string());

        let mut definitions = self.definitions.lock().unwrap();
        new_definitions.append(definitions.get_mut());
        definitions.set(new_definitions);

        Ok(())
    }

    /// Recursively goes through all the keys and convert them to `Vec<Definition>`
    fn parse_translation_structure(&self, value: &Value, json_path: String) -> Vec<Definition> {
        let mut definitions = vec![];

        match value {
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
                        key_filter_regex
                            .captures(&json_path.replace("\n", ""))
                            .and_then(|cap| cap.get(1).map(|group| group.as_str().to_string()))
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
                                .map(|matches| matches.as_str().to_string())
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
            _ => panic!("TODO: Error, translation file is not an object or string"),
        }

        definitions
    }

    /// Gets details about a single definition
    fn get_definition_detail_by_key(&self, key: &String) -> Option<String> {
        let definitions = self
            .definitions
            .lock()
            .unwrap()
            .get_mut()
            .iter()
            .filter(|definition| *definition == key)
            .map(|def| {
                format!(
                    "{}|**{}**|{}",
                    def.get_flag().unwrap_or_default(),
                    def.language.as_ref().unwrap_or(&"".to_string()),
                    def.value
                )
            })
            .intersperse("\n".to_string())
            .collect::<String>();

        if definitions.is_empty() {
            return None;
        };

        Some(format!("flag|language|translation\n-|-|-\n{}", definitions))
    }

    /// Fetches configuration
    async fn read_config(&self) {
        let config = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
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
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Incremental,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(
                        TRANSLATION_BEGIN_CHARS
                            .to_vec()
                            .iter()
                            .map(|char| char.to_string())
                            .collect(),
                    ),
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
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.read_config().await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        eprintln!("Shutting down...");
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::Info, "workspace folders changed!")
            .await;

        // TODO: Do not refetch configuration
        self.read_config().await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::Info, "configuration changed!")
            .await;

        // TODO: Do not refetch configuration but use params
        self.read_config().await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::Info, "watched files have changed!")
            .await;

        // TODO: Do not refetch configuration but use params
        self.read_config().await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "file opened!")
            .await;

        self.documents
            .lock()
            .unwrap()
            .get_mut()
            .push(FullTextDocument::new(
                params.text_document.uri,
                params.text_document.language_id,
                params.text_document.version.into(),
                params.text_document.text,
            ));
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.documents
            .lock()
            .unwrap()
            .get_mut()
            .iter_mut()
            .find(|doc| doc.uri == params.text_document.uri)
            .unwrap()
            .update(params.content_changes, params.text_document.version.into());
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "file closed!")
            .await;

        self.documents
            .lock()
            .unwrap()
            .get_mut()
            .retain(|document| document.uri != params.text_document.uri)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        let mut document = self
            .documents
            .lock()
            .unwrap()
            .get_mut()
            .iter_mut()
            .find(|document| document.uri == params.text_document_position.text_document.uri)
            .unwrap()
            .clone();

        let pos = document.offset_at(params.text_document_position.position);

        let range_result = get_editing_range(&document.text, &pos);
        if !range_result.is_some() {
            return Ok(None);
        };

        if let Ok(ref mut definitions) = self.definitions.try_lock() {
            let definitions = definitions.get_mut();
            let range = range_result.unwrap();

            Ok(Some(CompletionResponse::Array(
                definitions
                    .iter()
                    .unique_by(|definition| definition.get_identifier())
                    .map(|definition| CompletionItem {
                        label: definition.get_identifier().to_string(),
                        kind: Some(CompletionItemKind::Text),
                        detail: None,
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                            range: tower_lsp::lsp_types::Range::new(
                                document.position_at(range.start.try_into().unwrap()),
                                document.position_at(range.end.try_into().unwrap()),
                            ),
                            new_text: definition
                                .cleaned_key
                                .as_ref()
                                .unwrap_or(&definition.key)
                                .clone(),
                        })),
                        ..Default::default()
                    })
                    .collect(),
            )))
        } else {
            eprintln!("Gaat fout");
            Err(Error::internal_error())
        }
    }

    async fn completion_resolve(&self, params: CompletionItem) -> jsonrpc::Result<CompletionItem> {
        let mut item = params;
        if let Some(detail) = self.get_definition_detail_by_key(&item.label) {
            item.documentation = Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: detail,
            }));
        }

        Ok(item)
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let mut document = self
            .documents
            .lock()
            .unwrap()
            .get_mut()
            .iter_mut()
            .find(|document| document.uri == params.text_document_position_params.text_document.uri)
            .unwrap()
            .clone();

        let pos = document.offset_at(params.text_document_position_params.position);

        match find_translation_key_by_position(&document.text, &pos) {
            Some(translation_key) => {
                match self.get_definition_detail_by_key(&translation_key.as_str().to_string()) {
                    Some(contents) => {
                        let key_range = translation_key.range();

                        let range = tower_lsp::lsp_types::Range::new(
                            document.position_at(key_range.start.try_into().unwrap()),
                            document.position_at(key_range.end.try_into().unwrap()),
                        );

                        Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(contents)),
                            range: Some(range),
                        }))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(Backend::new);
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}

#[derive(Default, Debug)]
struct Definition {
    key: String,
    cleaned_key: Option<String>,
    file: String,
    language: Option<String>,
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

impl PartialEq<String> for Definition {
    fn eq(&self, other: &String) -> bool {
        match &self.cleaned_key {
            Some(cleaned_key) => cleaned_key == other,
            None => &self.key == other,
        }
    }
}

impl Definition {
    /// Returns the `cleaned_key` or the `key` if it does not exist.
    fn get_identifier(&self) -> &String {
        self.cleaned_key.as_ref().unwrap_or(&self.key)
    }

    /// Returns a flag emoji based on the supplied `language`
    fn get_flag(&self) -> Option<String> {
        let language = self.language.as_ref()?;

        // Splits 'en-us' to `vec!['en', 'us']`
        let mut possible_countries = language
            .split('-')
            .map(|text| text.to_uppercase())
            .collect_vec();

        possible_countries.push(language.to_uppercase());

        // Reverse it to prioritise `language`, then 'us', then 'en'
        possible_countries.reverse();

        for country in possible_countries {
            let flag = flag(&country);

            if flag.is_some() {
                return flag;
            }
        }
        None
    }
}

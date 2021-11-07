#[path = "./tests/backend.rs"]
#[cfg(test)]
mod tests;

#[path = "./tests/completion.rs"]
#[cfg(test)]
mod tests_completion;

#[path = "./tests/completion_yml.rs"]
#[cfg(test)]
mod tests_completion_yml;

#[path = "./tests/completion_multiple.rs"]
#[cfg(test)]
mod tests_completion_multiple;

#[path = "./tests/completion_exclude.rs"]
#[cfg(test)]
mod tests_completion_exclude;

#[path = "./tests/completion_invalid_translation_file.rs"]
#[cfg(test)]
mod tests_completion_invalid_translation_file;

#[path = "./tests/hover.rs"]
#[cfg(test)]
mod tests_hover;

#[path = "./tests/hover_per_language_file.rs"]
#[cfg(test)]
mod tests_hover_per_language_file;

#[path = "./tests/emoji_document.rs"]
#[cfg(test)]
mod tests_emoji_document;

mod full_text_document;
use crate::full_text_document::FullTextDocument;

use lsp_document::apply_change;
use lsp_document::{IndexedText, TextAdapter, TextMap};

mod string_helper;
use crate::string_helper::find_translation_key_by_position;
use country_emoji::flag;

use std::path::Path;
use string_helper::get_editing_range;
use string_helper::TRANSLATION_BEGIN_CHARS;
use string_helper::TRANSLATION_KEY_DIVIDER;

mod tree_sitter_helper;

use serde_json::json;
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

use std::fs;

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

#[derive(Deserialize, Debug, Default, Clone)]
struct TranslationFilesConfig {
    include: Vec<String>,
    exclude: Option<Vec<String>>,
}

impl TranslationFilesConfig {
    fn get_translation_files_from_patterns(
        &self,
        folders: &Vec<WorkspaceFolder>,
        patterns: &Vec<String>,
    ) -> Vec<PathBuf> {
        folders
            .iter()
            .map(|folder| {
                patterns
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
                                            Ok(path) => Some(path),
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
                    .collect::<Vec<PathBuf>>()
            })
            .flatten()
            .unique()
            .filter(|path| path.is_file())
            .collect()
    }

    fn get_translation_files_from_config(&self, folders: &Vec<WorkspaceFolder>) -> Vec<PathBuf> {
        let mut includes = self.get_translation_files_from_patterns(folders, &self.include);
        let excludes = match &self.exclude {
            Some(excludes) => self.get_translation_files_from_patterns(folders, excludes),
            None => vec![],
        };

        includes.retain(|file| !excludes.contains(file));
        includes
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
struct FileNameConfig {
    #[serde(with = "serde_regex")]
    details: Option<Regex>,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct KeyConfig {
    #[serde(with = "serde_regex", default)]
    details: Option<Regex>,
    #[serde(with = "serde_regex", default)]
    filter: Option<Regex>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConfig {
    translation_files: TranslationFilesConfig,
    file_name: FileNameConfig,
    #[serde(default)]
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

use std::ffi::OsStr;

use std::fmt;

#[derive(Debug)]
struct InvalidTranslationFileStructure;

impl fmt::Display for InvalidTranslationFileStructure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid translation file structure")
    }
}

impl std::error::Error for InvalidTranslationFileStructure {}

impl Backend {
    /// Figures out which translation files exists on the system of the user
    /// and calls `read_translation` to append them to the definitions
    async fn fetch_translations(&self, config_value: Value) {
        // TODO: Move setting config to other function
        let new_config: ExtensionConfig = serde_json::from_value(config_value).unwrap();
        self.config.lock().unwrap().set(new_config.clone());

        let folders = self.client.workspace_folders().await.unwrap().unwrap();

        self.client
            .log_message(MessageType::Info, format!("folders: {:?}", folders))
            .await;

        let files: Vec<PathBuf> = self
            .config
            .lock()
            .unwrap()
            .get_mut()
            .translation_files
            .get_translation_files_from_config(&folders);

        eprintln!("Translation files: {:?}", files);

        self.register_file_watch_capability(&new_config, &folders)
            .await;

        // Clear and add definitions
        self.definitions.lock().unwrap().set(vec![]);

        // TODO: Use self.client.log_message instead of eprintln!
        files.iter().for_each(|file| {
            match self.read_translation(file) {
                // TODO: Print this to VSCode
                Ok(_) => {
                    eprintln!("Loaded definitions from {:?}", file);

                    /* self.client
                    .log_message(MessageType::Info, format!("folders: {:?}", folders)).await; */
                }
                Err(err) => {
                    eprintln!("Could not read translation file {:?}.", file);
                    eprintln!("{:?}", err);

                    /* self.client
                    .log_message(MessageType::Info, format!("folders: {:?}", folders)).await; */
                }
            }
        });
    }

    /// Register capability to watch files
    async fn register_file_watch_capability(
        &self,
        config: &ExtensionConfig,
        folders: &Vec<WorkspaceFolder>,
    ) -> Result<(), Error> {
        let watched_patterns = DidChangeWatchedFilesRegistrationOptions {
            watchers: folders
                .iter()
                .map(|folder| {
                    config
                        .translation_files
                        .include
                        .iter()
                        .map(|pattern| {
                            FileSystemWatcher {
                                glob_pattern: path_clean::clean(
                                    folder
                                        .uri
                                        .to_file_path()
                                        .unwrap()
                                        .join(PathBuf::from(pattern))
                                        .to_str()
                                        .unwrap(),
                                ),
                                kind: None,
                            }
                        })
                        .collect::<Vec<FileSystemWatcher>>()
                })
                .flatten()
                .collect(),
        };

        self.client
            .register_capability(vec![Registration {
                id: "workspace/didChangeWatchedFiles".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(serde_json::to_value(watched_patterns).unwrap()),
            }])
            .await
    }

    async fn register_config_watch_capability(&self) -> Result<(), Error> {
        self.client
            .register_capability(vec![Registration {
                id: "workspace/didChangeConfiguration".to_string(),
                method: "workspace/didChangeConfiguration".to_string(),
                register_options: Some(
                    serde_json::to_value(DidChangeConfigurationParams {
                        settings: json!({ "lsp-translations": null }),
                    })
                    .unwrap(),
                ),
            }])
            .await
    }

    /// Reads the translations from a single file and adds them to the `definitions`
    fn read_translation(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::read_to_string(path)?;

        let ext = path.extension().and_then(OsStr::to_str);
        if ext.is_none() {
            return Err(Box::new(InvalidTranslationFileStructure));
        };

        let language = tree_sitter_helper::get_language_by_extension(ext.unwrap());
        if language.is_none() {
            return Err(Box::new(InvalidTranslationFileStructure));
        }

        let query_source = tree_sitter_helper::get_query_source_by_language(ext.unwrap());
        if query_source.is_none() {
            return Err(Box::new(InvalidTranslationFileStructure));
        }


        let new_definitions_result = tree_sitter_helper::parse_translation_structure(
            file,
            self.config.lock().unwrap().get_mut(),
            language.unwrap(),
            query_source.unwrap()
        );

        match new_definitions_result {
            Some(mut new_definitions) => {
                // Use file regex language for all above definitions
                let language = self
                    .config
                    .lock()
                    .unwrap()
                    .get_mut()
                    .file_name
                    .details
                    .as_ref()
                    .and_then(|file_name_details_regex| {
                        file_name_details_regex
                            .captures(path.file_name().unwrap().to_str().unwrap())
                            .and_then(|cap| {
                                cap.name("language")
                                    .map(|matches| matches.as_str().to_string())
                            })
                    });

                let translation_file = TranslationFile {
                    path: path.to_path_buf(),
                    language,
                };

                for definition in new_definitions.iter_mut() {
                    definition.file = Some(translation_file.clone());
                }

                let mut definitions = self.definitions.lock().unwrap();
                new_definitions.append(definitions.get_mut());
                definitions.set(new_definitions);

                Ok(())
            }
            None => Err(Box::new(InvalidTranslationFileStructure)),
        }
    }

    /// Gets details about a single definition
    fn get_definition_detail_by_key(&self, key: &String) -> Option<String> {
        if let Ok(ref mut definitions) = self.definitions.try_lock() {
            let definitions_same_key = definitions
                .get_mut()
                .iter()
                .filter(|definition| *definition == key);

            if definitions_same_key.clone().count() == 0 {
                return None;
            }

            let has_language = definitions_same_key
                .clone()
                .any(|definition| definition.get_language().is_some());

            let has_flag = definitions_same_key
                .clone()
                .any(|definition| definition.get_flag().is_some());

            let body = definitions_same_key
                .map(|def| {
                    if has_flag || has_language {
                        let row_data = vec![
                            def.get_flag().unwrap_or("🏴󠁢󠁳󠁢󠁰󠁿".to_string()),
                            format!("**{}**", def.get_language().unwrap_or(&"".to_string())),
                            def.get_printable_value(),
                        ];

                        row_data.join("|")
                    } else {
                        format!("|{}", def.get_printable_value())
                    }
                })
                .intersperse("\n".to_string())
                .collect::<String>();

            let header = if has_flag || has_language {
                "flag|language|translation\n-|-|-"
            } else {
                "|translation|\n|-"
            };
            return Some(format!("{}\n{}", header, body));
        }
        None
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
        let mut trigger_characters = TRANSLATION_BEGIN_CHARS
            .to_vec()
            .iter()
            .map(|char| char.to_string())
            .collect::<Vec<String>>();

        trigger_characters.push(TRANSLATION_KEY_DIVIDER.to_string());

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Incremental,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(trigger_characters),
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
        self.register_config_watch_capability().await;
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
        if let Ok(ref mut definitions) = self.documents.try_lock() {
            let documents = definitions.get_mut();

            let document = documents
                .iter_mut()
                .find(|doc| doc.uri == params.text_document.uri)
                .unwrap();

            for content_change in params.content_changes {
                let change = document.text.lsp_change_to_change(content_change).unwrap();

                document.text = IndexedText::new(apply_change(&document.text, change));
            }
        }
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
        let document = self
            .documents
            .lock()
            .unwrap()
            .get_mut()
            .iter_mut()
            .find(|document| document.uri == params.text_document_position.text_document.uri)
            .unwrap()
            .clone();

        let pos = document
            .text
            .lsp_pos_to_pos(&params.text_document_position.position)
            .unwrap();

        let range_result = get_editing_range(&document.text, &pos);
        if range_result.is_none() {
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
                                document.text.pos_to_lsp_pos(&range.start).unwrap(),
                                document.text.pos_to_lsp_pos(&range.end).unwrap(),
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
        let document = self
            .documents
            .lock()
            .unwrap()
            .get_mut()
            .iter_mut()
            .find(|document| document.uri == params.text_document_position_params.text_document.uri)
            .unwrap()
            .clone();

        let pos = document
            .text
            .lsp_pos_to_pos(&params.text_document_position_params.position)
            .unwrap();

        match find_translation_key_by_position(&document.text, &pos) {
            Some(translation_key) => {
                match self.get_definition_detail_by_key(&translation_key.as_str().to_string()) {
                    Some(contents) => {
                        let key_range = document
                            .text
                            .offset_range_to_range(translation_key.range())
                            .unwrap();

                        let range = tower_lsp::lsp_types::Range::new(
                            document.text.pos_to_lsp_pos(&key_range.start).unwrap(),
                            document.text.pos_to_lsp_pos(&key_range.end).unwrap(),
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

#[derive(Debug, Clone)]
struct TranslationFile {
    path: PathBuf,
    language: Option<String>,
}

#[derive(Default, Debug)]
pub struct Definition {
    key: String,
    cleaned_key: Option<String>,
    file: Option<TranslationFile>,
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

    fn get_language(&self) -> Option<&String> {
        return self
            .language
            .as_ref()
            .or(self.file.as_ref().and_then(|file| file.language.as_ref()));
    }

    /// Returns a flag emoji based on the supplied `language`
    fn get_flag(&self) -> Option<String> {
        let language = self.get_language()?;

        // Splits 'en-us' to `vec!['en', 'us']`
        let mut possible_countries = language
            .split('-')
            .map(|text| text.to_uppercase())
            .collect_vec();

        possible_countries.push(language.to_uppercase());

        if language.to_uppercase() == "EN" {
            possible_countries.push("US".to_string());
        }

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

    fn get_printable_value(&self) -> String {
        /* let newline_regex = Regex::new("\\n").unwrap();
        newline_regex.replace_all(&self.value, "<br />"); */
        self.value.escape_debug().to_string().replace("|", "\\|")
    }
}

#[path = "./tests/definition.rs"]
#[cfg(test)]
mod tests_definition;

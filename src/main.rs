#[path = "./tests/backend.rs"]
mod tests;

use serde_json::Value;
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use glob::glob;
use serde::Deserialize;
use std::borrow::BorrowMut;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate derive_new;

#[derive(Deserialize, Debug)]
struct FileNameConfig {
    details: String,
}

#[derive(Deserialize, Debug)]
struct KeyConfig {
    details: String,
    filter: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExtensionConfig {
    translation_files: Vec<String>,
    file_name: FileNameConfig,
    key: KeyConfig,
}

#[derive(Debug, new)]
pub struct Backend {
    client: Client,
    #[new(value = "vec![]")]
    definitions: Vec<Definition>,
}

impl Backend {
    async fn fetch_translations(&self, config_value: Value) {
        let config: ExtensionConfig = serde_json::from_value(config_value).unwrap();

        self.client
            .log_message(
                MessageType::Info,
                format!("read translations: {:?}", config),
            )
            .await;

        let folders = self.client.workspace_folders().await.unwrap().unwrap();

                self.client
            .log_message(MessageType::Info, format!("folders: {:?}", folders))
            .await;

        let files :Vec<PathBuf> = folders
            .iter()
            .map(|folder| {
                config
                    .translation_files
                    .iter()
                    .filter_map(|glob_pattern_setting| {
                        match Path::new(&folder.uri.path()).join(glob_pattern_setting).to_str() {
                            Some(glob_pattern) => match glob(glob_pattern) {
                                Ok(paths) => paths
                                    .map(|path| match path {
                                        Ok(path) => Some(path),
                                        Err(_) => {
                                            None
                                        }
                                    })
                                    .collect::<Option<PathBuf>>(),
                                Err(_) => {
                                    None
                                }
                            },
                            None => None,
                        }
                    }).collect::<PathBuf>()
            })
            .collect();

        self.client
            .log_message(MessageType::Info, format!("path bufs: {:?}", files))
            .await;
    }

    async fn read_translation() {

    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        initialize_params: InitializeParams,
    ) -> jsonrpc::Result<InitializeResult> {
        self.client
            .log_message(MessageType::Info, "initializing.....!")
            .await;

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Full,
                )),
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
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "initialized!")
            .await;

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
            }
            Err(err) => self.client.log_message(MessageType::Error, err).await,
        }
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::Info, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::Info, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::Info, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> jsonrpc::Result<Option<Value>> {
        self.client
            .log_message(MessageType::Info, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::Info, "applied").await,
            Ok(_) => self.client.log_message(MessageType::Info, "rejected").await,
            Err(err) => self.client.log_message(MessageType::Error, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "file opened!")
            .await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "file changed!")
            .await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "file closed!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> jsonrpc::Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::Info, "Completion!")
            .await;

        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
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

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
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

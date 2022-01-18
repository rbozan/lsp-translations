use lsp_document::IndexedText;
use tower_lsp::lsp_types::Url;

#[derive(Clone)]
pub struct FullTextDocument {
    pub uri: Url,

    /// The text document's language identifier.
    pub language_id: String,

    /// The version number of this document (it will strictly increase after each
    /// change, including undo/redo).
    pub version: i64,

    /// The content of the opened text document.
    pub text: IndexedText<String>,
}

impl FullTextDocument {
    pub fn new(uri: Url, language_id: String, version: i64, text: String) -> FullTextDocument {
        // let item = lsp_types::TextDocumentItem::new(uri, language_id, version, text);
        FullTextDocument {
            uri,
            language_id,
            version,
            text: IndexedText::new(text),
        }
    }
}

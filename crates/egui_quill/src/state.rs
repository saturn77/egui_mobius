//! Reactive state container for the editor citizen.

/// State for the `ReactiveEditor` panel — held inside a
/// `Dynamic<ReactiveEditorState>` so other panels and threads can
/// observe edits reactively.
#[derive(Clone, Default)]
pub struct ReactiveEditorState {
    pub content: String,
    pub language: String,
    pub theme: String,
}

impl ReactiveEditorState {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            language: "Plain Text".to_string(),
            theme: "base16-ocean.dark".to_string(),
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }
}

/// Languages exposed in the picker. Most names match syntect's
/// `default-syntaxes` set; OpenSCAD is bundled as a vendored
/// `.sublime-syntax` under this crate's `syntaxes/` directory and
/// merged into the SyntaxSet at first use.
pub const EDITOR_LANGUAGES: &[&str] = &[
    "Graphica",
    "Rust",
    "JSON",
    "YAML",
    "Python",
    "JavaScript",
    "Markdown",
    "OpenSCAD",
    "Plain Text",
];

/// Themes exposed in the picker. Names must match syntect's
/// `default-themes` set.
pub const EDITOR_THEMES: &[&str] = &[
    "base16-ocean.dark",
    "base16-ocean.light",
    "base16-eighties.dark",
    "base16-mocha.dark",
    "InspiredGitHub",
    "Solarized (dark)",
    "Solarized (light)",
];

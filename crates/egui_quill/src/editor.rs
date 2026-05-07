//! `ReactiveEditor` — the per-frame view onto a shared
//! `Dynamic<ReactiveEditorState>`. Syntax-highlighted text editor
//! with language and theme pickers as named atoms.

use egui_mobius_reactive::Dynamic;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::state::{ReactiveEditorState, EDITOR_LANGUAGES, EDITOR_THEMES};

/// Cache key — re-highlight only when content / language / theme change.
#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    content_hash: u64,
    language: String,
    theme: String,
}

thread_local! {
    /// Process-wide syntect tables. Loading defaults is non-trivial
    /// (~50ms cold) so we share a single set across all editor instances.
    static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static THEME_SET: ThemeSet = ThemeSet::load_defaults();
    /// Per-thread layout cache. Avoids re-highlighting on every frame
    /// when content/language/theme haven't changed.
    static LAYOUT_CACHE: Mutex<Option<(CacheKey, egui::text::LayoutJob)>> = const { Mutex::new(None) };
}

/// Per-frame view onto a `Dynamic<ReactiveEditorState>`. Construct
/// inside `ui` each frame, the way `ReactiveEventLogger::new` is used.
pub struct ReactiveEditor<'a> {
    state: &'a Dynamic<ReactiveEditorState>,
    show_pickers: bool,
}

impl<'a> ReactiveEditor<'a> {
    pub fn new(state: &'a Dynamic<ReactiveEditorState>) -> Self {
        Self {
            state,
            show_pickers: true,
        }
    }

    /// Toggle the in-panel language + theme pickers. Defaults to on.
    /// Consumer apps that surface those controls in their own menus
    /// or ribbons can call `.with_pickers(false)` so the editor body
    /// fills the entire panel without an internal toolbar.
    pub fn with_pickers(mut self, show: bool) -> Self {
        self.show_pickers = show;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.set_min_size(ui.available_size());

        let mut snap = self.state.get();

        // Atoms — language + theme pickers (optional).
        if self.show_pickers {
            ui.horizontal(|ui| {
                ui.label("Language:");
                egui::ComboBox::from_id_salt("egui_quill_language")
                    .selected_text(&snap.language)
                    .show_ui(ui, |ui| {
                        for option in EDITOR_LANGUAGES {
                            ui.selectable_value(&mut snap.language, option.to_string(), *option);
                        }
                    });

                ui.add_space(12.0);
                ui.label("Theme:");
                egui::ComboBox::from_id_salt("egui_quill_theme")
                    .selected_text(&snap.theme)
                    .show_ui(ui, |ui| {
                        for option in EDITOR_THEMES {
                            ui.selectable_value(&mut snap.theme, option.to_string(), *option);
                        }
                    });
            });
            ui.separator();
        }

        // Atom — the editable buffer.
        let language = snap.language.clone();
        let theme = snap.theme.clone();
        let mut buffer = std::mem::take(&mut snap.content);

        let mut layouter =
            move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap_width: f32| {
                let job = cached_layout(text.as_str(), &language, &theme);
                ui.ctx().fonts_mut(|f| f.layout_job(job))
            };

        let response = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut buffer)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                )
            });

        // Always write the current shape back — language/theme may
        // have changed via the pickers, content via TextEdit.
        snap.content = buffer;
        if response.inner.changed()
            || snap.language != self.state.get().language
            || snap.theme != self.state.get().theme
        {
            self.state.set(snap);
        }
    }
}

fn cached_layout(content: &str, language: &str, theme: &str) -> egui::text::LayoutJob {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let key = CacheKey {
        content_hash: hasher.finish(),
        language: language.to_string(),
        theme: theme.to_string(),
    };

    LAYOUT_CACHE.with(|cell| {
        let mut guard = cell.lock().unwrap();
        if let Some((cached_key, cached_job)) = &*guard {
            if cached_key == &key {
                return cached_job.clone();
            }
        }
        let job = build_layout(content, language, theme);
        *guard = Some((key, job.clone()));
        job
    })
}

fn build_layout(content: &str, language: &str, theme: &str) -> egui::text::LayoutJob {
    SYNTAX_SET.with(|syntax_set| {
        THEME_SET.with(|theme_set| {
            let syntax = syntax_set
                .find_syntax_by_name(language)
                .or_else(|| syntax_set.find_syntax_by_extension(language))
                .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

            let resolved_theme = theme_set
                .themes
                .get(theme)
                .or_else(|| theme_set.themes.get("base16-ocean.dark"))
                .expect("at least one theme available");

            let mut highlighter = HighlightLines::new(syntax, resolved_theme);
            let mut job = egui::text::LayoutJob::default();
            let monospace = egui::FontId::monospace(13.0);

            for line in LinesWithEndings::from(content) {
                let regions = match highlighter.highlight_line(line, syntax_set) {
                    Ok(r) => r,
                    Err(_) => {
                        job.append(
                            line,
                            0.0,
                            egui::TextFormat {
                                font_id: monospace.clone(),
                                color: egui::Color32::LIGHT_GRAY,
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                };
                for (style, segment) in regions {
                    job.append(segment, 0.0, syn_to_text_format(style, &monospace));
                }
            }
            job
        })
    })
}

fn syn_to_text_format(style: SynStyle, font_id: &egui::FontId) -> egui::TextFormat {
    let fg = style.foreground;
    let italics = style.font_style.contains(FontStyle::ITALIC);
    let underline = if style.font_style.contains(FontStyle::UNDERLINE) {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(fg.r, fg.g, fg.b))
    } else {
        egui::Stroke::NONE
    };
    egui::TextFormat {
        font_id: font_id.clone(),
        color: egui::Color32::from_rgb(fg.r, fg.g, fg.b),
        italics,
        underline,
        ..Default::default()
    }
}

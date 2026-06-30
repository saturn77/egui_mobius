//! Ribbon glyphs.
//!
//! These were Phosphor codepoints (`egui_phosphor::regular`); the crate now
//! draws from Unicode/emoji in egui's *default* fonts instead, so no icon font
//! has to be installed and there's no external version to keep in lockstep with
//! egui. The constant names match the Phosphor ones the ribbon already uses, so
//! call sites read `ico::RECTANGLE` exactly as before — only the rendered glyph
//! changed. Tweak any individual glyph here without touching the toolbar.

/// Selection / pointer tool.
pub const CURSOR: &str = "🖱";
/// Rectangle shape tool.
pub const RECTANGLE: &str = "▭";
/// Square shape tool.
pub const SQUARE: &str = "■";
/// Circle shape tool.
pub const CIRCLE: &str = "●";
/// Parallelogram shape tool.
pub const PARALLELOGRAM: &str = "▱";
/// Text label tool.
pub const TEXT_T: &str = "🔤";
/// Hierarchy / route tool.
pub const TREE_STRUCTURE: &str = "🌳";
/// Dashed-rectangle / marquee tool.
pub const RECTANGLE_DASHED: &str = "⬚";
/// Colour / palette.
pub const PALETTE: &str = "🎨";
/// Open folder.
pub const FOLDER_OPEN: &str = "📂";
/// Save.
pub const FLOPPY_DISK: &str = "💾";
/// File / document.
pub const FILE: &str = "📄";
/// Grid toggle.
pub const GRID_FOUR: &str = "▦";
/// Ruler / measure.
pub const RULER: &str = "📏";
/// Snap-dots.
pub const DOTS_NINE: &str = "⠿";
/// Snap-to-grid magnet.
pub const MAGNET: &str = "🧲";
/// Wire / line-segments.
pub const LINE_SEGMENTS: &str = "📐";
/// Fit-to-frame.
pub const FRAME_CORNERS: &str = "⛶";
/// Undo / reset.
pub const ARROW_COUNTER_CLOCKWISE: &str = "↺";
/// Keyboard shortcuts.
pub const KEYBOARD: &str = "⌨";

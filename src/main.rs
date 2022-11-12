extern crate core;

#[warn(clippy::all, clippy::pedantic)]
mod editor;
mod terminal;
mod document;
mod row;
mod highlighting;
mod filetype;

use editor::Editor;
pub use terminal::Terminal;
pub use editor::Position;
pub use document::Document;
pub use row::Row;
pub use editor::SearchDirection;
pub use filetype::HighlightingOptions;

fn main() {
    Editor::default().run();
}


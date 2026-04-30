//! Output rendering: JSON (default), text, or table.

use std::io::Write;

use clap::ValueEnum;
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Json,
    Text,
    Table,
}

/// Write a value in the requested format.
///
/// Types implementing `Serialize + Tabled + TextRender` can be rendered in all
/// three formats. The caller picks; JSON is always available.
pub fn render<W, T>(w: &mut W, fmt: Format, value: &T) -> std::io::Result<()>
where
    W: Write,
    T: Renderable,
{
    match fmt {
        Format::Json => value.render_json(w),
        Format::Text => value.render_text(w),
        Format::Table => value.render_table(w),
    }
}

/// Every top-level command response implements this.
pub trait Renderable {
    fn render_json<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
    fn render_text<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
    fn render_table<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

/// Blanket helper for rendering a slice of `Tabled + Serialize` rows.
pub fn render_rows<W, R>(
    w: &mut W,
    fmt: Format,
    rows: &[R],
    text_line: impl Fn(&R) -> String,
) -> std::io::Result<()>
where
    W: Write,
    R: Serialize + Tabled,
{
    match fmt {
        Format::Json => {
            serde_json::to_writer_pretty(&mut *w, rows)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            writeln!(w)
        }
        Format::Text => {
            for r in rows {
                writeln!(w, "{}", text_line(r))?;
            }
            Ok(())
        }
        Format::Table => {
            let mut table = Table::new(rows);
            table.with(Style::rounded());
            writeln!(w, "{table}")
        }
    }
}

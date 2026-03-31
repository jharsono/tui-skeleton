use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{AnimationMode, cell_intensity, interpolate_color, is_uniform};
use crate::defaults;

/// Deterministic cell width fractions — cycles across (row, col) pairs.
const DEFAULT_CELL_WIDTHS: [f32; 11] = [
    0.45, 0.70, 0.30, 0.85, 0.55, 0.40, 0.75, 0.60, 0.35, 0.50, 0.65,
];

/// Skeleton table with rows, column separators, and optional zebra striping.
///
/// Column widths are specified as [`Constraint`] slices, matching how
/// ratatui tables define their layouts. Each cell fills only a fraction
/// of its column width (set via [`cell_widths`](SkeletonTable::cell_widths)),
/// mimicking ragged text content.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonTable<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    braille: bool,
    base: Color,
    highlight: Color,
    rows: u16,
    columns: &'a [Constraint],
    cell_widths: &'a [f32],
    zebra: bool,
    block: Option<ratatui_widgets::block::Block<'a>>,
}

/// Brightness offset applied to odd rows when zebra striping is enabled.
const ZEBRA_OFFSET: f32 = 0.15;

impl<'a> SkeletonTable<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            braille: false,
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            rows: 5,
            columns: &[],
            cell_widths: &DEFAULT_CELL_WIDTHS,
            zebra: true,
            block: None,
        }
    }

    pub fn mode(mut self, mode: AnimationMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn braille(mut self, braille: bool) -> Self {
        self.braille = braille;
        self
    }

    pub fn base(mut self, color: impl Into<Color>) -> Self {
        self.base = color.into();
        self
    }

    pub fn highlight(mut self, color: impl Into<Color>) -> Self {
        self.highlight = color.into();
        self
    }

    pub fn rows(mut self, rows: u16) -> Self {
        self.rows = rows;
        self
    }

    pub fn columns(mut self, columns: &'a [Constraint]) -> Self {
        self.columns = columns;
        self
    }

    /// Per-cell width fractions (`0.0..=1.0`) cycling across `(row, col)`.
    ///
    /// Each cell renders only this fraction of its column width,
    /// producing ragged edges that mimic real text data. Default
    /// uses a built-in 11-element pattern.
    pub fn cell_widths(mut self, widths: &'a [f32]) -> Self {
        self.cell_widths = widths;
        self
    }

    /// Enable or disable alternating row brightness. Default: `true`.
    pub fn zebra(mut self, zebra: bool) -> Self {
        self.zebra = zebra;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() {
            return;
        }

        let col_offsets = resolve_columns(self.columns, inner.width);
        let col_ranges = column_ranges(&col_offsets, inner.width);
        let num_cols = col_ranges.len().max(1);

        let uniform_t = is_uniform(self.mode)
            .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width));

        let row_count = self.rows.min(inner.height);

        for row in 0..row_count {
            let y = inner.y + row;

            let zebra_boost = if self.zebra && row % 2 == 1 {
                ZEBRA_OFFSET
            } else {
                0.0
            };

            // Render separators.
            for &sep in &col_offsets {
                let x = inner.x + sep;

                buf[(x, y)]
                    .set_char('│')
                    .set_style(Style::default().fg(self.base));
            }

            // Render cell content with per-cell width variation.
            for (ci, &(start, width)) in col_ranges.iter().enumerate() {
                let cell_idx = row as usize * num_cols + ci;
                let frac = self.cell_widths[cell_idx % self.cell_widths.len()].clamp(0.0, 1.0);
                let fill_width = ((width as f32) * frac).ceil() as u16;

                for dx in 0..fill_width.min(width) {
                    let col = start + dx;
                    let x = inner.x + col;

                    let t = uniform_t.unwrap_or_else(|| {
                        cell_intensity(self.mode, self.elapsed_ms, col, inner.width)
                    });
                    let t = (t + zebra_boost).min(1.0);
                    let fg = interpolate_color(self.base, self.highlight, self.mode, t);
                    let glyph = crate::animation::cell_glyph(
                        self.braille,
                        self.mode,
                        self.elapsed_ms,
                        row,
                        col,
                    );

                    buf[(x, y)]
                        .set_char(glyph)
                        .set_style(Style::default().fg(fg));
                }
            }
        }
    }
}

/// Resolve `Constraint` slices to absolute column boundary offsets.
///
/// Returns the x-offsets where column separators should appear (between columns).
fn resolve_columns(constraints: &[Constraint], width: u16) -> Vec<u16> {
    if constraints.is_empty() {
        return Vec::new();
    }

    let total_seps = constraints.len().saturating_sub(1) as u16;
    let available = width.saturating_sub(total_seps);

    let widths: Vec<u16> = constraints
        .iter()
        .map(|c| match c {
            Constraint::Length(n) | Constraint::Min(n) | Constraint::Max(n) => (*n).min(available),
            Constraint::Percentage(p) => (available as u32 * (*p).min(100) as u32 / 100) as u16,
            Constraint::Ratio(num, den) => {
                if *den == 0 {
                    0
                } else {
                    (available as u32 * *num / *den) as u16
                }
            }
            Constraint::Fill(_) => 0,
        })
        .collect();

    // Distribute remaining space to Fill columns, or evenly if none specified.
    let allocated: u16 = widths.iter().sum();
    let remaining = available.saturating_sub(allocated);
    let fill_count = constraints
        .iter()
        .filter(|c| matches!(c, Constraint::Fill(_)))
        .count() as u16;

    let widths: Vec<u16> = if fill_count > 0 {
        let fill_each = remaining / fill_count.max(1);

        widths
            .iter()
            .zip(constraints)
            .map(|(w, c)| {
                if matches!(c, Constraint::Fill(_)) {
                    fill_each
                } else {
                    *w
                }
            })
            .collect()
    } else {
        widths
    };

    // Convert widths to separator offsets.
    let mut offsets = Vec::with_capacity(constraints.len().saturating_sub(1));
    let mut x = 0u16;

    for (i, w) in widths.iter().enumerate() {
        x += w;

        if i < widths.len() - 1 {
            offsets.push(x);
            x += 1; // separator column
        }
    }

    offsets
}

/// Derive `(start_col, width)` ranges for each column from separator offsets.
fn column_ranges(offsets: &[u16], total_width: u16) -> Vec<(u16, u16)> {
    if offsets.is_empty() {
        return vec![(0, total_width)];
    }

    let mut ranges = Vec::with_capacity(offsets.len() + 1);
    let mut start = 0u16;

    for &sep in offsets {
        ranges.push((start, sep.saturating_sub(start)));
        start = sep + 1; // skip separator
    }

    ranges.push((start, total_width.saturating_sub(start)));
    ranges
}

#[cfg(feature = "pantry")]
#[path = "table.ingredient.rs"]
pub mod ingredient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_correct_row_count() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);

        SkeletonTable::new(1000).rows(3).render(area, &mut buf);

        // Row 3 (0-indexed) should be empty.
        assert_ne!(buf[(0, 0)].symbol(), " ");
        assert_ne!(buf[(0, 2)].symbol(), " ");
        assert_eq!(buf[(0, 3)].symbol(), " ");
    }

    #[test]
    fn column_separators_present() {
        let cols = [Constraint::Length(5), Constraint::Length(5)];
        let area = Rect::new(0, 0, 11, 3);
        let mut buf = Buffer::empty(area);

        SkeletonTable::new(1000)
            .columns(&cols)
            .rows(3)
            .render(area, &mut buf);

        // Separator at column 5.
        assert_eq!(buf[(5, 0)].symbol(), "│");
    }

    #[test]
    fn cells_have_ragged_widths() {
        let cols = [Constraint::Length(10), Constraint::Length(10)];
        let area = Rect::new(0, 0, 21, 2);
        let mut buf = Buffer::empty(area);

        // Force predictable widths: first cell 50%, second cell 100%.
        SkeletonTable::new(1000)
            .columns(&cols)
            .rows(2)
            .cell_widths(&[0.5, 1.0])
            .zebra(false)
            .render(area, &mut buf);

        // First cell of row 0: 50% of 10 = 5 filled, cell 5 should be empty.
        assert_ne!(buf[(0, 0)].symbol(), " ");
        assert_eq!(buf[(5, 0)].symbol(), " "); // unfilled tail

        // Second cell of row 0: 100% of 10 = all filled.
        assert_ne!(buf[(11, 0)].symbol(), " "); // after separator at col 10
        assert_ne!(buf[(20, 0)].symbol(), " ");
    }

    #[test]
    fn empty_area_is_noop() {
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let expected = buf.clone();

        SkeletonTable::new(0).render(area, &mut buf);

        assert_eq!(buf, expected);
    }

    #[test]
    fn resolve_percentage_columns() {
        let constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let offsets = resolve_columns(&constraints, 21);

        // 21 - 1 separator = 20 available; 50% each = 10; separator at offset 10.
        assert_eq!(offsets, vec![10]);
    }
}

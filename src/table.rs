use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{cell_intensity, interpolate_color, AnimationMode};
use crate::defaults;

/// Skeleton table with rows, column separators, and optional zebra striping.
///
/// Column widths are specified as [`Constraint`] slices, matching how
/// ratatui tables define their layouts.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonTable<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    base: Color,
    highlight: Color,
    rows: u16,
    columns: &'a [Constraint],
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
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            rows: 5,
            columns: &[],
            zebra: true,
            block: None,
        }
    }

    pub fn mode(mut self, mode: AnimationMode) -> Self {
        self.mode = mode;
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

        // Breathe is uniform — hoist outside the loop.
        let breathe_t = matches!(self.mode, AnimationMode::Breathe)
            .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width));

        let row_count = self.rows.min(inner.height);

        for row in 0..row_count {
            let y = inner.y + row;

            let zebra_boost = if self.zebra && row % 2 == 1 {
                ZEBRA_OFFSET
            } else {
                0.0
            };

            for col in 0..inner.width {
                let x = inner.x + col;

                // Column separators.
                if is_separator(&col_offsets, col) {
                    buf[(x, y)]
                        .set_char('│')
                        .set_style(Style::default().fg(self.base));
                    continue;
                }

                let t = breathe_t.unwrap_or_else(|| {
                    cell_intensity(self.mode, self.elapsed_ms, col, inner.width)
                });
                let t = (t + zebra_boost).min(1.0);
                let fg = interpolate_color(self.base, self.highlight, self.mode, t);

                buf[(x, y)].set_char('█').set_style(Style::default().fg(fg));
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

fn is_separator(offsets: &[u16], col: u16) -> bool {
    offsets.contains(&col)
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

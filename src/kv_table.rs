use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{cell_intensity, interpolate_color, AnimationMode};
use crate::defaults;

/// Deterministic value width fractions cycling across rows.
const DEFAULT_VALUE_WIDTHS: [f32; 5] = [0.60, 0.40, 0.75, 0.35, 0.55];

/// Skeleton key-value table (properties panel / detail view).
///
/// Renders pairs of short fixed-width keys on the left and
/// variable-width values on the right, separated by a dim `│`.
/// Each pair occupies one row with a gap row between pairs.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonKvTable<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    base: Color,
    highlight: Color,
    pairs: u16,
    key_width: u16,
    value_widths: &'a [f32],
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonKvTable<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            pairs: 5,
            key_width: 12,
            value_widths: &DEFAULT_VALUE_WIDTHS,
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

    /// Number of key-value pairs. Default: `5`.
    pub fn pairs(mut self, pairs: u16) -> Self {
        self.pairs = pairs;
        self
    }

    /// Fixed width of the key column in cells. Default: `12`.
    pub fn key_width(mut self, width: u16) -> Self {
        self.key_width = width;
        self
    }

    /// Per-pair value width fractions (`0.0..=1.0`) of the remaining space.
    ///
    /// The pattern cycles when there are more pairs than entries.
    pub fn value_widths(mut self, widths: &'a [f32]) -> Self {
        self.value_widths = widths;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonKvTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        // key_width + separator(1) + gap(1) + at least 1 value cell
        if inner.is_empty() || inner.width < self.key_width + 3 || self.value_widths.is_empty() {
            return;
        }

        let sep_col = self.key_width;
        let value_start = sep_col + 2; // separator + 1 gap
        let value_space = inner.width - value_start;

        let stride = 2u16; // content row + gap row
        let pair_count = self.pairs.min((inner.height + 1) / stride);

        // Breathe is uniform — hoist.
        let breathe_t = matches!(self.mode, AnimationMode::Breathe)
            .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width));

        for i in 0..pair_count {
            let y = inner.y + i * stride;

            if y >= inner.bottom() {
                break;
            }

            // Key cells.
            for col in 0..self.key_width {
                let x = inner.x + col;
                let t = breathe_t.unwrap_or_else(|| {
                    cell_intensity(self.mode, self.elapsed_ms, col, inner.width)
                });
                let fg = interpolate_color(self.base, self.highlight, self.mode, t);

                buf[(x, y)].set_char('█').set_style(Style::default().fg(fg));
            }

            // Separator.
            buf[(inner.x + sep_col, y)]
                .set_char('│')
                .set_style(Style::default().fg(self.base));

            // Value cells.
            let frac = self.value_widths[i as usize % self.value_widths.len()].clamp(0.0, 1.0);
            let val_width = ((value_space as f32) * frac).ceil() as u16;

            for col in 0..val_width.min(value_space) {
                let abs_col = value_start + col;
                let x = inner.x + abs_col;
                let t = breathe_t.unwrap_or_else(|| {
                    cell_intensity(self.mode, self.elapsed_ms, abs_col, inner.width)
                });
                let fg = interpolate_color(self.base, self.highlight, self.mode, t);

                buf[(x, y)].set_char('█').set_style(Style::default().fg(fg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_between_key_and_value() {
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);

        SkeletonKvTable::new(1000)
            .pairs(1)
            .key_width(8)
            .render(area, &mut buf);

        // Key fills cols 0..8.
        assert_eq!(buf[(0, 0)].symbol(), "█");
        assert_eq!(buf[(7, 0)].symbol(), "█");

        // Separator at col 8.
        assert_eq!(buf[(8, 0)].symbol(), "│");

        // Gap at col 9, value starts at col 10.
        assert_eq!(buf[(9, 0)].symbol(), " ");
        assert_eq!(buf[(10, 0)].symbol(), "█");
    }

    #[test]
    fn pairs_have_gaps() {
        let area = Rect::new(0, 0, 30, 4);
        let mut buf = Buffer::empty(area);

        SkeletonKvTable::new(1000)
            .pairs(2)
            .key_width(5)
            .render(area, &mut buf);

        // Row 0: content.
        assert_eq!(buf[(0, 0)].symbol(), "█");

        // Row 1: gap.
        assert_eq!(buf[(0, 1)].symbol(), " ");

        // Row 2: content.
        assert_eq!(buf[(0, 2)].symbol(), "█");
    }

    #[test]
    fn too_narrow_is_noop() {
        let area = Rect::new(0, 0, 5, 5);
        let mut buf = Buffer::empty(area);
        let expected = buf.clone();

        SkeletonKvTable::new(1000)
            .key_width(10)
            .render(area, &mut buf);

        assert_eq!(buf, expected);
    }
}

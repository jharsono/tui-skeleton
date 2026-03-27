use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{cell_intensity, interpolate_color, AnimationMode};
use crate::defaults;

/// Deterministic width fractions cycling across bars.
const DEFAULT_WIDTHS: [f32; 7] = [0.85, 0.60, 0.95, 0.45, 0.75, 0.55, 0.70];

/// Skeleton horizontal bar chart with bars of varying length.
///
/// Renders rows of `█` extending from the left edge, separated by
/// single-row gaps. Widths cycle deterministically. Mimics a
/// leaderboard or ranking display.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonHBarChart<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    base: Color,
    highlight: Color,
    bars: u16,
    bar_height: u16,
    widths: &'a [f32],
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonHBarChart<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            bars: 5,
            bar_height: 1,
            widths: &DEFAULT_WIDTHS,
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

    /// Number of horizontal bars. Default: `5`.
    pub fn bars(mut self, bars: u16) -> Self {
        self.bars = bars;
        self
    }

    /// Height of each bar in rows. Default: `1`.
    pub fn bar_height(mut self, height: u16) -> Self {
        self.bar_height = height;
        self
    }

    /// Override the per-bar width fractions (`0.0..=1.0`).
    ///
    /// The pattern cycles when there are more bars than entries.
    pub fn widths(mut self, widths: &'a [f32]) -> Self {
        self.widths = widths;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonHBarChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() || self.widths.is_empty() || self.bar_height == 0 {
            return;
        }

        let stride = self.bar_height + 1; // bar + 1-row gap
        let bar_count = self.bars.min((inner.height + 1) / stride);

        // Breathe is uniform — hoist.
        let breathe_t = matches!(self.mode, AnimationMode::Breathe)
            .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width));

        for i in 0..bar_count {
            let frac = self.widths[i as usize % self.widths.len()].clamp(0.0, 1.0);
            let bar_width = ((inner.width as f32) * frac).ceil() as u16;
            let bar_y = inner.y + i * stride;

            for dy in 0..self.bar_height {
                let y = bar_y + dy;

                if y >= inner.bottom() {
                    break;
                }

                for col in 0..bar_width.min(inner.width) {
                    let x = inner.x + col;

                    let t = breathe_t.unwrap_or_else(|| {
                        cell_intensity(self.mode, self.elapsed_ms, col, inner.width)
                    });
                    let fg = interpolate_color(self.base, self.highlight, self.mode, t);

                    buf[(x, y)].set_char('█').set_style(Style::default().fg(fg));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_extend_from_left() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);

        SkeletonHBarChart::new(1000)
            .bars(1)
            .bar_height(1)
            .widths(&[0.5])
            .render(area, &mut buf);

        // 50% of 20 = 10 cells.
        assert_eq!(buf[(9, 0)].symbol(), "█");
        assert_eq!(buf[(10, 0)].symbol(), " ");
    }

    #[test]
    fn bars_have_gaps() {
        let area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(area);

        SkeletonHBarChart::new(1000)
            .bars(2)
            .bar_height(1)
            .widths(&[1.0, 1.0])
            .render(area, &mut buf);

        // Bar 0: row 0. Gap: row 1. Bar 1: row 2.
        assert_eq!(buf[(0, 0)].symbol(), "█");
        assert_eq!(buf[(0, 1)].symbol(), " ");
        assert_eq!(buf[(0, 2)].symbol(), "█");
    }

    #[test]
    fn multi_row_bars() {
        let area = Rect::new(0, 0, 10, 7);
        let mut buf = Buffer::empty(area);

        SkeletonHBarChart::new(1000)
            .bars(2)
            .bar_height(2)
            .widths(&[1.0, 1.0])
            .render(area, &mut buf);

        // Bar 0: rows 0-1. Gap: row 2. Bar 1: rows 3-4.
        assert_eq!(buf[(0, 0)].symbol(), "█");
        assert_eq!(buf[(0, 1)].symbol(), "█");
        assert_eq!(buf[(0, 2)].symbol(), " ");
        assert_eq!(buf[(0, 3)].symbol(), "█");
        assert_eq!(buf[(0, 4)].symbol(), "█");
    }
}

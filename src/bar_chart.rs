use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{cell_intensity, interpolate_color, AnimationMode};
use crate::defaults;

/// Deterministic height fractions cycling across bars.
const DEFAULT_HEIGHTS: [f32; 7] = [0.6, 0.85, 0.45, 0.95, 0.70, 0.55, 0.80];

/// Skeleton vertical bar chart with bars of varying height.
///
/// Renders columns of `█` rising from the bottom of the area,
/// separated by single-cell gaps. Heights cycle deterministically.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonBarChart<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    base: Color,
    highlight: Color,
    bars: u16,
    bar_width: u16,
    heights: &'a [f32],
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonBarChart<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            bars: 6,
            bar_width: 3,
            heights: &DEFAULT_HEIGHTS,
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

    /// Number of bars to render. Default: `6`.
    pub fn bars(mut self, bars: u16) -> Self {
        self.bars = bars;
        self
    }

    /// Width of each bar in cells. Default: `3`.
    pub fn bar_width(mut self, width: u16) -> Self {
        self.bar_width = width;
        self
    }

    /// Override the per-bar height fractions (`0.0..=1.0`).
    ///
    /// The pattern cycles when there are more bars than entries.
    pub fn heights(mut self, heights: &'a [f32]) -> Self {
        self.heights = heights;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonBarChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() || self.heights.is_empty() || self.bar_width == 0 {
            return;
        }

        let stride = self.bar_width + 1; // bar + 1-cell gap
        let bar_count = self.bars.min((inner.width + 1) / stride);

        // Breathe is uniform — hoist.
        let breathe_t = matches!(self.mode, AnimationMode::Breathe)
            .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width));

        for i in 0..bar_count {
            let frac = self.heights[i as usize % self.heights.len()].clamp(0.0, 1.0);
            let bar_height = ((inner.height as f32) * frac).ceil() as u16;
            let bar_x = inner.x + i * stride;
            let bar_top = inner.y + inner.height - bar_height;

            for dy in 0..bar_height {
                let y = bar_top + dy;

                for dx in 0..self.bar_width {
                    let x = bar_x + dx;

                    if x >= inner.right() {
                        break;
                    }

                    let col = x - inner.x;
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
    fn bars_rise_from_bottom() {
        let area = Rect::new(0, 0, 10, 10);
        let mut buf = Buffer::empty(area);

        SkeletonBarChart::new(1000)
            .bars(1)
            .bar_width(2)
            .heights(&[0.5])
            .render(area, &mut buf);

        // 50% of 10 = 5 rows. Top cell of bar at row 5.
        assert_eq!(buf[(0, 5)].symbol(), "█");
        assert_eq!(buf[(1, 5)].symbol(), "█");

        // Row 4 should be empty (above the bar).
        assert_eq!(buf[(0, 4)].symbol(), " ");

        // Bottom row should be filled.
        assert_eq!(buf[(0, 9)].symbol(), "█");
    }

    #[test]
    fn bars_have_gaps() {
        let area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(area);

        SkeletonBarChart::new(1000)
            .bars(2)
            .bar_width(2)
            .heights(&[1.0, 1.0])
            .render(area, &mut buf);

        // Bar 0: cols 0-1. Bar 1: cols 3-4. Col 2 is the gap.
        assert_eq!(buf[(0, 0)].symbol(), "█");
        assert_eq!(buf[(1, 0)].symbol(), "█");
        assert_eq!(buf[(2, 0)].symbol(), " ");
        assert_eq!(buf[(3, 0)].symbol(), "█");
    }

    #[test]
    fn overflow_bars_clipped() {
        let area = Rect::new(0, 0, 5, 5);
        let mut buf = Buffer::empty(area);

        // bar_width=3, stride=4. Only 1 bar fits in width 5.
        SkeletonBarChart::new(1000)
            .bars(3)
            .bar_width(3)
            .heights(&[1.0])
            .render(area, &mut buf);

        assert_eq!(buf[(0, 0)].symbol(), "█");
        assert_eq!(buf[(2, 0)].symbol(), "█");

        // Second bar would start at col 4, only 1 cell fits.
        // bar_count is min(3, (5+1)/4=1) = 1, so second bar not rendered.
    }
}

use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{AnimationMode, cell_glyph, cell_intensity, interpolate_color, is_uniform};
use crate::defaults;

/// Braille characters for rounded-cap progress bars.
const BAR_FULL: &str = "\u{28FF}"; // ⣿
const BAR_ROUND_LEFT: &str = "\u{28BE}"; // ⢾
const BAR_ROUND_RIGHT: &str = "\u{2877}"; // ⡷

/// Deterministic fill fractions cycling across bars.
const DEFAULT_FILLS: [f32; 5] = [0.62, 0.85, 0.38, 0.74, 0.50];

/// Braille progress bar with rounded end caps and optional peak marker.
///
/// Renders single-row bars of braille characters (`⢾⣿⣿⣿⡷`) with
/// configurable fill level and an optional peak marker. Multiple bars
/// stack vertically with single-row gaps — useful as a skeleton for
/// CPU meters, memory gauges, or any horizontal gauge display.
///
/// In [`AnimationMode::Noise`], structural glyphs are replaced with
/// random braille dot patterns that change every frame.
///
/// Inspired by [ratatui-braille-bar](https://github.com/penso/ratatui-braille-bar).
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonBrailleBar<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    base: Color,
    highlight: Color,
    empty: Color,
    bars: u16,
    fills: &'a [f32],
    peak: Option<f32>,
    peak_color: Option<Color>,
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonBrailleBar<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            empty: Color::Rgb(60, 60, 60),
            bars: 3,
            fills: &DEFAULT_FILLS,
            peak: None,
            peak_color: None,
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

    /// Color for empty (unfilled) cells. Default: dark gray `#3C3C3C`.
    pub fn empty(mut self, color: impl Into<Color>) -> Self {
        self.empty = color.into();
        self
    }

    /// Number of stacked bars. Default: `3`.
    pub fn bars(mut self, bars: u16) -> Self {
        self.bars = bars;
        self
    }

    /// Override the per-bar fill fractions (`0.0..=1.0`).
    ///
    /// The pattern cycles when there are more bars than entries.
    pub fn fills(mut self, fills: &'a [f32]) -> Self {
        self.fills = fills;
        self
    }

    /// Fractional position for a peak marker on each bar.
    ///
    /// Rendered as a single cell at the given fraction of bar width,
    /// clamped to at least the fill position. Set to `None` to disable.
    pub fn peak(mut self, peak: f32) -> Self {
        self.peak = Some(peak);
        self
    }

    /// Color for the peak marker cell.
    ///
    /// Defaults to highlight color when unset.
    pub fn peak_color(mut self, color: impl Into<Color>) -> Self {
        self.peak_color = Some(color.into());
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonBrailleBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() || self.fills.is_empty() {
            return;
        }

        let width = inner.width as usize;
        let stride: u16 = 2; // bar + 1-row gap
        let bar_count = self.bars.min((inner.height + 1) / stride);

        let uniform_t = is_uniform(self.mode)
            .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width));
        let noise = self.mode == AnimationMode::Noise;

        for i in 0..bar_count {
            let y = inner.y + i * stride;
            let row = y - inner.y;

            if y >= inner.bottom() {
                break;
            }

            let frac = self.fills[i as usize % self.fills.len()].clamp(0.0, 1.0);
            let filled = ((frac * width as f32) as usize).min(width);

            let peak_pos = self.peak.map(|p| {
                let pos = (p.clamp(0.0, 1.0) * width as f32) as usize;
                pos.max(filled.saturating_sub(1))
                    .min(width.saturating_sub(1))
            });

            for col_idx in 0..width {
                let col = col_idx as u16;
                let x = inner.x + col;

                if noise {
                    let ch = cell_glyph(false, self.mode, self.elapsed_ms, row, col);
                    let t = uniform_t.unwrap_or(0.0);
                    let fg = interpolate_color(self.base, self.highlight, self.mode, t);

                    buf[(x, y)].set_char(ch).set_style(Style::default().fg(fg));
                    continue;
                }

                let glyph = match col_idx {
                    0 => BAR_ROUND_LEFT,
                    n if n == width - 1 => BAR_ROUND_RIGHT,
                    _ => BAR_FULL,
                };

                let fg = if peak_pos == Some(col_idx) {
                    self.peak_color.unwrap_or(self.highlight)
                } else if col_idx < filled {
                    let t = uniform_t.unwrap_or_else(|| {
                        cell_intensity(self.mode, self.elapsed_ms, col, inner.width)
                    });
                    interpolate_color(self.base, self.highlight, self.mode, t)
                } else {
                    self.empty
                };

                buf[(x, y)]
                    .set_symbol(glyph)
                    .set_style(Style::default().fg(fg));
            }
        }
    }
}

#[cfg(feature = "pantry")]
#[path = "braille_bar.ingredient.rs"]
pub mod ingredient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_braille_characters() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);

        SkeletonBrailleBar::new(1000)
            .bars(1)
            .fills(&[1.0])
            .render(area, &mut buf);

        let has_braille = (0..20).any(|x| {
            let sym = buf[(x as u16, 0u16)].symbol();
            sym.chars()
                .next()
                .is_some_and(|c| (0x2800..=0x28FF).contains(&(c as u32)))
        });

        assert!(has_braille, "expected braille characters in output");
    }

    #[test]
    fn rounded_caps() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);

        SkeletonBrailleBar::new(1000)
            .bars(1)
            .fills(&[1.0])
            .render(area, &mut buf);

        assert_eq!(buf[(0, 0)].symbol(), "⢾");
        assert_eq!(buf[(9, 0)].symbol(), "⡷");
    }

    #[test]
    fn filled_cells_use_animation_color() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);

        SkeletonBrailleBar::new(0)
            .bars(1)
            .fills(&[0.5])
            .base(Color::Rgb(10, 20, 30))
            .empty(Color::Rgb(60, 60, 60))
            .render(area, &mut buf);

        // At elapsed_ms=0, Breathe intensity=0 → filled cells get base color.
        assert_eq!(buf[(1, 0)].style().fg, Some(Color::Rgb(10, 20, 30)));

        // Unfilled cells get empty color.
        assert_eq!(buf[(9, 0)].style().fg, Some(Color::Rgb(60, 60, 60)));
    }

    #[test]
    fn bars_have_gaps() {
        let area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(area);

        SkeletonBrailleBar::new(1000)
            .bars(2)
            .fills(&[1.0, 1.0])
            .render(area, &mut buf);

        // Bar 0: row 0. Gap: row 1. Bar 1: row 2.
        assert_ne!(buf[(0, 0)].symbol(), " ");
        assert_eq!(buf[(0, 1)].symbol(), " ");
        assert_ne!(buf[(0, 2)].symbol(), " ");
    }

    #[test]
    fn peak_marker_position() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);

        SkeletonBrailleBar::new(1000)
            .bars(1)
            .fills(&[0.3])
            .peak(0.7)
            .peak_color(Color::Rgb(251, 146, 60))
            .render(area, &mut buf);

        assert_eq!(buf[(7, 0)].style().fg, Some(Color::Rgb(251, 146, 60)));
    }

    #[test]
    fn noise_mode_fills_random_braille() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);

        SkeletonBrailleBar::new(1000)
            .mode(AnimationMode::Noise)
            .bars(1)
            .fills(&[1.0])
            .render(area, &mut buf);

        for x in 0..10u16 {
            let ch = buf[(x, 0)].symbol().chars().next().unwrap();
            assert!((0x2800..=0x28FF).contains(&(ch as u32)));
        }

        // Caps should NOT be structural — all random.
        assert_ne!(buf[(0, 0)].symbol(), "⢾");
    }

    #[test]
    fn empty_area_is_noop() {
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let expected = buf.clone();

        SkeletonBrailleBar::new(0).render(area, &mut buf);

        assert_eq!(buf, expected);
    }
}

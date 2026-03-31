use ratatui_core::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::animation::AnimationMode;
use crate::block::render_skeleton_cells;
use crate::defaults;

/// Default line-width fractions simulating paragraph variance.
const DEFAULT_LINE_WIDTHS: [f32; 7] = [0.90, 0.85, 0.92, 0.78, 0.88, 0.80, 0.55];

/// Lines never exceed this fraction of the container width.
const MAX_WIDTH_FRAC: f32 = 0.95;

/// Default duration for the typewriter fill to complete.
const DEFAULT_DURATION_MS: u64 = 3000;

/// Skeleton simulating streaming chat text.
///
/// Cells fill left-to-right, top-to-bottom over `duration_ms`, mimicking
/// text being typed into a chat bubble. Once the fill completes, all
/// lines remain visible and the animation mode pulses normally.
///
/// Line widths are deterministic fractions that cycle, producing a
/// ragged paragraph shape.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonStreamingText<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    braille: bool,
    base: Color,
    highlight: Color,
    lines: u16,
    duration_ms: u64,
    repeat: bool,
    line_widths: &'a [f32],
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonStreamingText<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            braille: false,
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            lines: 5,
            duration_ms: DEFAULT_DURATION_MS,
            repeat: false,
            line_widths: &DEFAULT_LINE_WIDTHS,
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

    /// Total lines of text to fill. Default: `5`.
    pub fn lines(mut self, lines: u16) -> Self {
        self.lines = lines;
        self
    }

    /// Duration in milliseconds for the typewriter to fill all lines. Default: `3000`.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Loop the typewriter fill. When true, the fill resets after
    /// `duration_ms` plus a hold period and replays. Default: `false`.
    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    /// Per-line width fractions (`0.0..=1.0`), cycling for more lines.
    pub fn line_widths(mut self, widths: &'a [f32]) -> Self {
        self.line_widths = widths;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonStreamingText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() || self.line_widths.is_empty() || self.lines == 0 {
            return;
        }

        let total_width = inner.width;
        let render_lines = self.lines.min(inner.height);
        let widths = self.line_widths;

        // Compute per-line cell counts, then total cells across all lines.
        let line_cells: Vec<u16> = (0..render_lines)
            .map(|row| {
                let frac = widths[row as usize % widths.len()].clamp(0.0, MAX_WIDTH_FRAC);
                (total_width as f32 * frac) as u16
            })
            .collect();

        let total_cells: u64 = line_cells.iter().map(|&w| w as u64).sum();

        // When repeating, cycle through fill + hold, then reset.
        let hold_ms = self.duration_ms * 2 / 3;
        let cycle_ms = self.duration_ms + hold_ms;
        let effective_ms = if self.repeat {
            self.elapsed_ms % cycle_ms
        } else {
            self.elapsed_ms
        };

        let progress = if self.duration_ms == 0 || effective_ms >= self.duration_ms {
            total_cells
        } else {
            total_cells * effective_ms / self.duration_ms
        };

        // Cumulative cell count — determines which cells are filled.
        let mut cumulative = 0u64;
        let mut line_starts: Vec<u64> = Vec::with_capacity(render_lines as usize);

        for &cells in &line_cells {
            line_starts.push(cumulative);
            cumulative += cells as u64;
        }

        render_skeleton_cells(
            Rect::new(inner.x, inner.y, inner.width, render_lines),
            buf,
            self.mode,
            self.braille,
            self.elapsed_ms,
            self.base,
            self.highlight,
            |row, col, _width| {
                let row_idx = row as usize;

                if row_idx >= line_cells.len() {
                    return false;
                }

                let line_width = line_cells[row_idx];

                if col >= line_width {
                    return false;
                }

                // Cell's absolute position in the typewriter sequence.
                let cell_pos = line_starts[row_idx] + col as u64;
                cell_pos < progress
            },
        );
    }
}

#[cfg(feature = "pantry")]
#[path = "streaming_text.ingredient.rs"]
pub mod ingredient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cells_at_zero() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);

        SkeletonStreamingText::new(0)
            .lines(5)
            .duration_ms(1000)
            .render(area, &mut buf);

        for y in 0..5 {
            for x in 0..20 {
                assert_eq!(buf[(x, y)].symbol(), " ");
            }
        }
    }

    #[test]
    fn all_cells_after_duration() {
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);

        // 0.95 cap → 19 of 20 cols filled per line.
        SkeletonStreamingText::new(5000)
            .lines(3)
            .duration_ms(1000)
            .line_widths(&[1.0])
            .render(area, &mut buf);

        for y in 0..3u16 {
            assert_eq!(buf[(18, y)].symbol(), "█");
            assert_eq!(buf[(19, y)].symbol(), " ");
        }
    }

    #[test]
    fn partial_fill_first_line() {
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        // 0.5 × 20 = 10 cols per line, 2 lines = 20 total cells.
        // At 500/1000 = 50%, 10 cells filled = first line.
        SkeletonStreamingText::new(500)
            .lines(2)
            .duration_ms(1000)
            .line_widths(&[0.5])
            .render(area, &mut buf);

        // First line fully filled (10 cols).
        for x in 0..10 {
            assert_eq!(buf[(x, 0u16)].symbol(), "█");
        }

        // Second line empty.
        for x in 0..10 {
            assert_eq!(buf[(x, 1u16)].symbol(), " ");
        }
    }

    #[test]
    fn ragged_widths_respected() {
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        // Line 0: 50% = 10 cols, line 1: 90% = 18 cols (under 0.95 cap).
        SkeletonStreamingText::new(2000)
            .lines(2)
            .duration_ms(1000)
            .line_widths(&[0.5, 0.9])
            .render(area, &mut buf);

        // Line 0: cols 0-9 filled, col 10 empty.
        assert_eq!(buf[(9, 0u16)].symbol(), "█");
        assert_eq!(buf[(10, 0u16)].symbol(), " ");

        // Line 1: col 17 filled, col 18 empty.
        assert_eq!(buf[(17, 1u16)].symbol(), "█");
        assert_eq!(buf[(18, 1u16)].symbol(), " ");
    }
}

use ratatui_core::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::animation::AnimationMode;
use crate::block::render_skeleton_cells;
use crate::defaults;

/// Default paragraph simulation: two full lines, one shorter, repeat.
const DEFAULT_LINE_WIDTHS: [f32; 5] = [1.0, 1.0, 0.80, 1.0, 0.60];

/// Skeleton paragraph with lines of varying width.
///
/// Simulates a block of text by rendering `█` characters at different
/// widths per line. The pattern cycles when the area has more rows
/// than entries in `line_widths`.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonText<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    braille: bool,
    base: Color,
    highlight: Color,
    line_widths: &'a [f32],
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonText<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            braille: false,
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
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

    /// Per-line width fractions (`0.0..=1.0`), cycling for longer areas.
    pub fn line_widths(mut self, widths: &'a [f32]) -> Self {
        self.line_widths = widths;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() || self.line_widths.is_empty() {
            return;
        }

        let widths = self.line_widths;
        let total_width = inner.width;

        render_skeleton_cells(
            inner,
            buf,
            self.mode,
            self.braille,
            self.elapsed_ms,
            self.base,
            self.highlight,
            |row, col, _width| {
                let frac = widths[row as usize % widths.len()].clamp(0.0, 1.0);
                let line_width = (total_width as f32 * frac) as u16;
                col < line_width
            },
        );
    }
}

#[cfg(feature = "pantry")]
#[path = "text.ingredient.rs"]
pub mod ingredient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pattern_varies_width() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);

        SkeletonText::new(1000).render(area, &mut buf);

        // Line 0: 100% → col 19 filled.
        assert_eq!(buf[(19, 0)].symbol(), "█");

        // Line 2: 80% → col 16 empty (80% of 20 = 16).
        assert_eq!(buf[(15, 2)].symbol(), "█");
        assert_eq!(buf[(16, 2)].symbol(), " ");

        // Line 4: 60% → col 12 empty (60% of 20 = 12).
        assert_eq!(buf[(11, 4)].symbol(), "█");
        assert_eq!(buf[(12, 4)].symbol(), " ");
    }

    #[test]
    fn custom_line_widths() {
        let area = Rect::new(0, 0, 10, 2);
        let mut buf = Buffer::empty(area);

        SkeletonText::new(1000)
            .line_widths(&[0.5, 1.0])
            .render(area, &mut buf);

        // Line 0: 50% → col 5 empty.
        assert_eq!(buf[(4, 0)].symbol(), "█");
        assert_eq!(buf[(5, 0)].symbol(), " ");

        // Line 1: 100% → col 9 filled.
        assert_eq!(buf[(9, 1)].symbol(), "█");
    }
}

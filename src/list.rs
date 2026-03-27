use ratatui_core::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::animation::AnimationMode;
use crate::block::render_skeleton_cells;
use crate::defaults;

/// Deterministic width fractions cycling across items to simulate list variance.
const DEFAULT_WIDTHS: [f32; 5] = [0.45, 0.30, 0.55, 0.35, 0.50];

/// Skeleton list with short, spaced items and ragged right edges.
///
/// Each item is a single row separated by a blank gap row, mimicking
/// a menu or sidebar. Width fractions cycle deterministically.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonList<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    base: Color,
    highlight: Color,
    items: u16,
    widths: &'a [f32],
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonList<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            items: 5,
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

    /// Number of list items to display. Default: `5`.
    pub fn items(mut self, items: u16) -> Self {
        self.items = items;
        self
    }

    /// Override the per-item width fractions (`0.0..=1.0`).
    ///
    /// The pattern cycles when there are more items than entries.
    pub fn widths(mut self, widths: &'a [f32]) -> Self {
        self.widths = widths;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.is_empty() || self.widths.is_empty() {
            return;
        }

        // Each item occupies 1 row of content + 1 row of gap.
        let rows_needed = self.items * 2;
        let render_height = rows_needed.min(inner.height);
        let widths = self.widths;
        let total_width = inner.width;

        render_skeleton_cells(
            Rect::new(inner.x, inner.y, inner.width, render_height),
            buf,
            self.mode,
            self.elapsed_ms,
            self.base,
            self.highlight,
            |row, col, _width| {
                // Odd rows are gaps.
                if row % 2 == 1 {
                    return false;
                }

                let item_index = (row / 2) as usize;
                let frac = widths[item_index % widths.len()].clamp(0.0, 1.0);
                let item_width = (total_width as f32 * frac) as u16;
                col < item_width
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_have_gaps() {
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = Buffer::empty(area);

        SkeletonList::new(1000)
            .items(3)
            .widths(&[0.5, 0.5, 0.5])
            .render(area, &mut buf);

        // Row 0: item content.
        assert_eq!(buf[(0, 0)].symbol(), "█");

        // Row 1: gap.
        assert_eq!(buf[(0, 1)].symbol(), " ");

        // Row 2: item content.
        assert_eq!(buf[(0, 2)].symbol(), "█");

        // Row 3: gap.
        assert_eq!(buf[(0, 3)].symbol(), " ");
    }

    #[test]
    fn ragged_edges() {
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = Buffer::empty(area);

        SkeletonList::new(1000)
            .items(3)
            .widths(&[0.5, 0.3, 0.8])
            .render(area, &mut buf);

        // Item 0 at 50% → col 9 filled, col 10 empty.
        assert_eq!(buf[(9, 0)].symbol(), "█");
        assert_eq!(buf[(10, 0)].symbol(), " ");

        // Item 1 at 30% → col 5 filled, col 6 empty.
        assert_eq!(buf[(5, 2)].symbol(), "█");
        assert_eq!(buf[(6, 2)].symbol(), " ");
    }

    #[test]
    fn respects_item_limit() {
        let area = Rect::new(0, 0, 10, 10);
        let mut buf = Buffer::empty(area);

        SkeletonList::new(1000).items(2).render(area, &mut buf);

        // 2 items = rows 0,2 filled; row 4 should be empty.
        assert_ne!(buf[(0, 0)].symbol(), " ");
        assert_ne!(buf[(0, 2)].symbol(), " ");
        assert_eq!(buf[(0, 4)].symbol(), " ");
    }
}

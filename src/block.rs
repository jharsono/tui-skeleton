use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{AnimationMode, cell_glyph, cell_intensity, interpolate_color, is_uniform};
use crate::defaults;

/// Solid filled rectangle with animated brightness.
///
/// The atomic skeleton unit — fills every cell at a color interpolated
/// between `base` and `highlight` according to the chosen
/// [`AnimationMode`]. [`braille`](SkeletonBlock::braille) switches fill
/// from `█` to `⣿`; [`AnimationMode::Noise`] uses random braille glyphs.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonBlock<'a> {
    elapsed_ms: u64,
    mode: AnimationMode,
    braille: bool,
    base: Color,
    highlight: Color,
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonBlock<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            mode: AnimationMode::default(),
            braille: false,
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            block: None,
        }
    }

    pub fn mode(mut self, mode: AnimationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Use random braille dot patterns instead of solid `█` fill.
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

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SkeletonBlock<'_> {
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

        render_skeleton_cells(
            inner,
            buf,
            self.mode,
            self.braille,
            self.elapsed_ms,
            self.base,
            self.highlight,
            |_row, col, width| col < width,
        );
    }
}

/// Fill cells in `area` where `visible(row, col, width)` returns true.
///
/// Shared by all skeleton widget shapes. `braille: true` renders `⣿`.
/// [`AnimationMode::Noise`] renders random braille glyphs.
#[expect(clippy::too_many_arguments)]
pub(crate) fn render_skeleton_cells(
    area: Rect,
    buf: &mut Buffer,
    mode: AnimationMode,
    braille: bool,
    elapsed_ms: u64,
    base: Color,
    highlight: Color,
    visible: impl Fn(u16, u16, u16) -> bool,
) {
    let uniform_t = is_uniform(mode).then(|| cell_intensity(mode, elapsed_ms, 0, area.width));

    for row in 0..area.height {
        for col in 0..area.width {
            if !visible(row, col, area.width) {
                continue;
            }

            let t = uniform_t.unwrap_or_else(|| cell_intensity(mode, elapsed_ms, col, area.width));
            let fg = interpolate_color(base, highlight, mode, t);
            let ch = cell_glyph(braille, mode, elapsed_ms, row, col);

            let cell = &mut buf[(area.x + col, area.y + row)];
            cell.set_char(ch);
            cell.set_style(Style::default().fg(fg));
        }
    }
}

#[cfg(feature = "pantry")]
#[path = "block.ingredient.rs"]
pub mod ingredient;

#[cfg(test)]
mod tests {
    use super::*;

    fn render_block(elapsed_ms: u64, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);

        SkeletonBlock::new(elapsed_ms).render(area, &mut buf);

        buf
    }

    #[test]
    fn fills_all_cells() {
        let buf = render_block(1000, 10, 3);

        for y in 0..3 {
            for x in 0..10 {
                assert_eq!(buf[(x, y)].symbol(), "█");
            }
        }
    }

    #[test]
    fn noise_mode_fills_random_braille() {
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);

        SkeletonBlock::new(1000)
            .mode(AnimationMode::Noise)
            .render(area, &mut buf);

        for y in 0..3u16 {
            for x in 0..10u16 {
                let ch = buf[(x, y)].symbol().chars().next().unwrap();
                assert!((0x2800..=0x28FF).contains(&(ch as u32)));
            }
        }
    }

    #[test]
    fn braille_flag_fills_solid_braille() {
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);

        SkeletonBlock::new(1000)
            .braille(true)
            .render(area, &mut buf);

        for y in 0..3u16 {
            for x in 0..10u16 {
                assert_eq!(buf[(x, y)].symbol(), "⣿");
            }
        }
    }

    #[test]
    fn empty_area_is_noop() {
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let expected = buf.clone();

        SkeletonBlock::new(0).render(area, &mut buf);

        assert_eq!(buf, expected);
    }

    #[test]
    fn custom_colors_applied() {
        let area = Rect::new(0, 0, 5, 1);
        let mut buf = Buffer::empty(area);

        SkeletonBlock::new(0)
            .base(Color::Rgb(10, 20, 30))
            .highlight(Color::Rgb(200, 200, 200))
            .render(area, &mut buf);

        // At elapsed_ms=0, Breathe intensity is 0.0 → all cells should be base color.
        for x in 0..5 {
            let style = buf[(x, 0u16)].style();
            assert_eq!(style.fg, Some(Color::Rgb(10, 20, 30)));
        }
    }
}

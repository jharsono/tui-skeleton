use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::animation::{AnimationMode, cell_intensity, interpolate_color, is_uniform};
use crate::defaults;

/// Braille dot offsets within a 2×4 cell.
///
/// ```text
/// (0,0)=0x01  (1,0)=0x08
/// (0,1)=0x02  (1,1)=0x10
/// (0,2)=0x04  (1,2)=0x20
/// (0,3)=0x40  (1,3)=0x80
/// ```
const DOT: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];

/// Braille blank character (U+2800).
const BRAILLE_BLANK: u32 = 0x2800;

/// Deterministic wave amplitudes for layered lines.
const DEFAULT_AMPLITUDES: [f32; 3] = [0.7, 0.45, 0.85];

/// Deterministic frequency multipliers for layered lines.
const DEFAULT_FREQUENCIES: [f32; 3] = [1.0, 1.7, 0.6];

/// Deterministic vertical offsets for layered lines.
const DEFAULT_OFFSETS: [f32; 3] = [0.5, 0.35, 0.65];

/// Wave drift speed: one full pixel-width scroll per 20s.
const DRIFT_PERIOD_MS: f32 = 20_000.0;

/// Skeleton line chart rendered with braille traces over filled area.
///
/// Generates deterministic sine-wave paths that drift over time,
/// rendered as braille dot traces with solid `█` fill below each
/// line. The filled area makes skeleton animations (Breathe, Sweep,
/// Plasma) clearly visible, while the braille edge gives the chart
/// its line-chart silhouette.
#[must_use]
#[derive(Debug, Clone)]
pub struct SkeletonLineChart<'a> {
    elapsed_ms: u64,
    drift_ms: Option<u64>,
    mode: AnimationMode,
    braille: bool,
    base: Color,
    highlight: Color,
    lines: u16,
    filled: bool,
    block: Option<ratatui_widgets::block::Block<'a>>,
}

impl<'a> SkeletonLineChart<'a> {
    pub fn new(elapsed_ms: u64) -> Self {
        Self {
            elapsed_ms,
            drift_ms: None,
            mode: AnimationMode::default(),
            braille: false,
            base: defaults::BASE,
            highlight: defaults::HIGHLIGHT,
            lines: 2,
            filled: true,
            block: None,
        }
    }

    /// Override the timestamp used for wave drift.
    ///
    /// When set, the wave shape is computed from this fixed value
    /// while color animation still uses `elapsed_ms`. Pass `0` to
    /// freeze the wave in place.
    pub fn drift_ms(mut self, drift_ms: u64) -> Self {
        self.drift_ms = Some(drift_ms);
        self
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

    /// Number of overlapping line traces. Default: `2`.
    pub fn lines(mut self, lines: u16) -> Self {
        self.lines = lines;
        self
    }

    /// Fill the area below each line with `█`. Default: `true`.
    ///
    /// When enabled, the filled region carries the skeleton animation
    /// (Breathe/Sweep/Plasma) while the braille trace sits on top as
    /// the edge. Disable for line-only rendering.
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

/// Bundles animation parameters shared across rendering passes.
struct Coloring {
    mode: AnimationMode,
    braille: bool,
    elapsed_ms: u64,
    base: Color,
    highlight: Color,
    breathe_t: Option<f32>,
}

impl Coloring {
    fn color_at(&self, col: u16, width: u16) -> Color {
        let t = self
            .breathe_t
            .unwrap_or_else(|| cell_intensity(self.mode, self.elapsed_ms, col, width));
        interpolate_color(self.base, self.highlight, self.mode, t)
    }
}

impl Widget for SkeletonLineChart<'_> {
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

        let pixel_w = inner.width as usize * 2;
        let pixel_h = inner.height as usize * 4;
        let line_count = self.lines.min(DEFAULT_AMPLITUDES.len() as u16) as usize;

        let drift_time = self.drift_ms.unwrap_or(self.elapsed_ms);
        let drift = drift_time as f32 / DRIFT_PERIOD_MS * std::f32::consts::TAU;

        // Track the highest wave (lowest y-pixel) at each column for fill.
        let mut fill_top = vec![pixel_h; pixel_w];

        // Build braille dot grid for the line traces.
        let mut dots = vec![vec![false; pixel_w]; pixel_h];

        for line_idx in 0..line_count {
            let amplitude = DEFAULT_AMPLITUDES[line_idx];
            let freq = DEFAULT_FREQUENCIES[line_idx];
            let offset = DEFAULT_OFFSETS[line_idx];
            let line_drift = drift * (1.0 + line_idx as f32 * 0.3);

            plot_wave(
                &mut dots,
                &mut fill_top,
                pixel_w,
                pixel_h,
                amplitude,
                freq,
                offset,
                line_drift,
            );
        }

        let coloring = Coloring {
            mode: self.mode,
            braille: self.braille,
            elapsed_ms: self.elapsed_ms,
            base: self.base,
            highlight: self.highlight,
            breathe_t: is_uniform(self.mode)
                .then(|| cell_intensity(self.mode, self.elapsed_ms, 0, inner.width)),
        };

        if self.filled {
            render_fill(inner, buf, &fill_top, pixel_h, &coloring);
        }

        render_braille(inner, buf, &dots, pixel_w, pixel_h, &coloring);
    }
}

/// Render solid `█` fill from each column's wave-top down to the bottom.
fn render_fill(
    inner: Rect,
    buf: &mut Buffer,
    fill_top: &[usize],
    pixel_h: usize,
    color: &Coloring,
) {
    for cx in 0..inner.width as usize {
        let top_pixel = fill_top
            .get(cx * 2)
            .copied()
            .unwrap_or(pixel_h)
            .min(fill_top.get(cx * 2 + 1).copied().unwrap_or(pixel_h));

        let fill_start = top_pixel / 4 + 1;
        let col = cx as u16;
        let fg = color.color_at(col, inner.width);

        for cy in fill_start..inner.height as usize {
            let x = inner.x + col;
            let y = inner.y + cy as u16;
            let glyph = crate::animation::cell_glyph(
                color.braille,
                color.mode,
                color.elapsed_ms,
                cy as u16,
                col,
            );

            buf[(x, y)]
                .set_char(glyph)
                .set_style(Style::default().fg(fg));
        }
    }
}

/// Encode dot grid into braille characters with animation color.
fn render_braille(
    inner: Rect,
    buf: &mut Buffer,
    dots: &[Vec<bool>],
    pixel_w: usize,
    pixel_h: usize,
    color: &Coloring,
) {
    for cy in 0..inner.height as usize {
        for cx in 0..inner.width as usize {
            let mut pattern: u8 = 0;

            for (dx, dot_col) in DOT.iter().enumerate() {
                for (dy, &bit) in dot_col.iter().enumerate() {
                    let px = cx * 2 + dx;
                    let py = cy * 4 + dy;

                    if px < pixel_w && py < pixel_h && dots[py][px] {
                        pattern |= bit;
                    }
                }
            }

            if pattern == 0 {
                continue;
            }

            let braille = char::from_u32(BRAILLE_BLANK + pattern as u32).unwrap_or('⠀');
            let col = cx as u16;
            let fg = color.color_at(col, inner.width);

            let x = inner.x + col;
            let y = inner.y + cy as u16;
            buf[(x, y)]
                .set_symbol(&braille.to_string())
                .set_style(Style::default().fg(fg));
        }
    }
}

/// Plot a drifting sine wave onto the dot grid and update fill envelope.
#[expect(clippy::too_many_arguments)]
fn plot_wave(
    dots: &mut [Vec<bool>],
    fill_top: &mut [usize],
    pixel_w: usize,
    pixel_h: usize,
    amplitude: f32,
    freq: f32,
    offset: f32,
    drift: f32,
) {
    let mut prev_y: Option<usize> = None;

    for px in 0..pixel_w {
        let phase = px as f32 / pixel_w as f32 * std::f32::consts::TAU * freq + drift;
        let normalized = offset + amplitude * 0.5 * phase.sin();
        let py = ((1.0 - normalized.clamp(0.0, 1.0)) * (pixel_h - 1) as f32) as usize;

        dots[py][px] = true;
        fill_top[px] = fill_top[px].min(py);

        // Connect to previous pixel vertically to avoid gaps.
        if let Some(prev) = prev_y {
            let (lo, hi) = if prev < py { (prev, py) } else { (py, prev) };

            for row in dots.iter_mut().take(hi + 1).skip(lo) {
                row[px] = true;
            }
        }

        prev_y = Some(py);
    }
}

#[cfg(feature = "pantry")]
#[path = "line_chart.ingredient.rs"]
pub mod ingredient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_braille_characters() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);

        SkeletonLineChart::new(1000).lines(1).render(area, &mut buf);

        let has_braille = (0..20)
            .flat_map(|x| (0..5).map(move |y| (x, y)))
            .any(|(x, y)| {
                let sym = buf[(x as u16, y as u16)].symbol();
                sym.chars()
                    .next()
                    .is_some_and(|c| (0x2800..=0x28FF).contains(&(c as u32)))
            });

        assert!(has_braille, "expected braille characters in output");
    }

    #[test]
    fn filled_area_below_wave() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);

        SkeletonLineChart::new(1000)
            .lines(1)
            .filled(true)
            .render(area, &mut buf);

        // Bottom row should be filled (wave never sits at the very bottom).
        let bottom_filled = (0..20).any(|x| buf[(x as u16, 9u16)].symbol() == "█");
        assert!(bottom_filled, "bottom row should have fill");
    }

    #[test]
    fn unfilled_has_no_blocks() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);

        SkeletonLineChart::new(1000)
            .lines(1)
            .filled(false)
            .render(area, &mut buf);

        let has_block = (0..20)
            .flat_map(|x| (0..10).map(move |y| (x, y)))
            .any(|(x, y)| buf[(x as u16, y as u16)].symbol() == "█");

        assert!(!has_block, "unfilled mode should have no █ characters");
    }

    #[test]
    fn drift_changes_output() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf_a = Buffer::empty(area);
        let mut buf_b = Buffer::empty(area);

        SkeletonLineChart::new(0).lines(1).render(area, &mut buf_a);
        SkeletonLineChart::new(5000)
            .lines(1)
            .render(area, &mut buf_b);

        assert_ne!(
            buf_a, buf_b,
            "different timestamps should produce different output"
        );
    }

    #[test]
    fn empty_area_is_noop() {
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let expected = buf.clone();

        SkeletonLineChart::new(0).render(area, &mut buf);

        assert_eq!(buf, expected);
    }
}

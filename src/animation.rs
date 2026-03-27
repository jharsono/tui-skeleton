use ratatui_core::style::Color;

/// Animation style for skeleton loading widgets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AnimationMode {
    /// Single brightness sweep left-to-right, then rest.
    Sweep,
    /// Uniform pulse: entire bar fades between dim and bright.
    #[default]
    Breathe,
    /// Two sine waves at different frequencies drift in opposite directions,
    /// creating organic shifting brightness patterns.
    Plasma,
}

// ── Timing constants ────────────────────────────────────────────────

/// Sweep: 800ms travel + 2s rest.
const SWEEP_MS: u64 = 800;
const SWEEP_CYCLE_MS: u64 = SWEEP_MS + 2000;

/// Half-width of the cosine brightness window (cells from center).
const SHIMMER_RADIUS: f32 = 12.0;

/// Breathe: 5s full sine cycle.
const BREATHE_CYCLE_MS: u64 = 5000;

/// Plasma wave parameters.
const PLASMA_PERIOD_MS: f32 = 4000.0;
const PLASMA_AMPLITUDE: f32 = 0.6;
const PLASMA_FREQ_A: f32 = 0.18;
const PLASMA_FREQ_B: f32 = 0.29;

// ── Intensity ───────────────────────────────────────────────────────

/// Compute animation intensity for a single cell.
///
/// Returns a value in `[0.0, 1.0]` representing brightness progression
/// from `base` toward `highlight`.
pub fn cell_intensity(mode: AnimationMode, elapsed_ms: u64, col: u16, width: u16) -> f32 {
    match mode {
        AnimationMode::Sweep => sweep_intensity(elapsed_ms, col, width),
        AnimationMode::Breathe => breathe_intensity(elapsed_ms),
        AnimationMode::Plasma => plasma_intensity(elapsed_ms, col),
    }
}

fn sweep_intensity(elapsed_ms: u64, col: u16, width: u16) -> f32 {
    let phase = elapsed_ms % SWEEP_CYCLE_MS;

    if phase >= SWEEP_MS {
        return 0.0;
    }

    let width = width as f32;
    let sweep_span = width + SHIMMER_RADIUS * 2.0;
    let progress = phase as f32 / SWEEP_MS as f32;
    let center = -SHIMMER_RADIUS + progress * sweep_span;
    let dist = (col as f32 - center).abs();

    if dist >= SHIMMER_RADIUS {
        0.0
    } else {
        (1.0 + (dist / SHIMMER_RADIUS * std::f32::consts::PI).cos()) * 0.5
    }
}

fn breathe_intensity(elapsed_ms: u64) -> f32 {
    let phase = (elapsed_ms % BREATHE_CYCLE_MS) as f32 / BREATHE_CYCLE_MS as f32;
    (phase * std::f32::consts::TAU).sin().abs()
}

fn plasma_intensity(elapsed_ms: u64, col: u16) -> f32 {
    let time = elapsed_ms as f32 / PLASMA_PERIOD_MS * std::f32::consts::TAU;
    let x = col as f32;

    let wave_a = (x * PLASMA_FREQ_A + time).sin();
    let wave_b = (x * PLASMA_FREQ_B - time * 0.7).sin();

    ((wave_a + wave_b) * 0.25 + 0.5) * PLASMA_AMPLITUDE
}

// ── Color interpolation ─────────────────────────────────────────────

/// Interpolate between `base` and `highlight` at the given intensity.
///
/// For [`AnimationMode::Plasma`], the highlight is extrapolated 2× past
/// the base→highlight distance so peaks are clearly visible.
pub fn interpolate_color(
    base: Color,
    highlight: Color,
    mode: AnimationMode,
    intensity: f32,
) -> Color {
    let (br, bg, bb) = rgb_components(base);
    let (hr, hg, hb) = rgb_components(highlight);

    // Plasma doubles the contrast range.
    let (pr, pg, pb) = if mode == AnimationMode::Plasma {
        (
            hr.saturating_add(hr.saturating_sub(br)),
            hg.saturating_add(hg.saturating_sub(bg)),
            hb.saturating_add(hb.saturating_sub(bb)),
        )
    } else {
        (hr, hg, hb)
    };

    Color::Rgb(
        lerp_u8(br, pr, intensity),
        lerp_u8(bg, pg, intensity),
        lerp_u8(bb, pb, intensity),
    )
}

fn rgb_components(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::DarkGray => (128, 128, 128),
        Color::Gray => (169, 169, 169),
        Color::White => (255, 255, 255),
        Color::Black => (0, 0, 0),
        _ => (128, 128, 128),
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breathe_zero_starts_at_zero() {
        assert_eq!(breathe_intensity(0), 0.0);
    }

    #[test]
    fn breathe_quarter_cycle_peaks() {
        let intensity = breathe_intensity(BREATHE_CYCLE_MS / 4);
        assert!((intensity - 1.0).abs() < 0.01);
    }

    #[test]
    fn sweep_rest_phase_is_zero() {
        assert_eq!(sweep_intensity(SWEEP_MS + 100, 5, 40), 0.0);
    }

    #[test]
    fn plasma_stays_bounded() {
        for col in 0..80 {
            let t = plasma_intensity(1234, col);
            assert!((0.0..=1.0).contains(&t), "plasma out of bounds: {t}");
        }
    }

    #[test]
    fn interpolate_at_zero_returns_base() {
        let base = Color::Rgb(10, 20, 30);
        let highlight = Color::Rgb(100, 200, 255);
        let result = interpolate_color(base, highlight, AnimationMode::Breathe, 0.0);
        assert_eq!(result, Color::Rgb(10, 20, 30));
    }

    #[test]
    fn interpolate_at_one_returns_highlight() {
        let base = Color::Rgb(0, 0, 0);
        let highlight = Color::Rgb(100, 100, 100);
        let result = interpolate_color(base, highlight, AnimationMode::Breathe, 1.0);
        assert_eq!(result, Color::Rgb(100, 100, 100));
    }

    #[test]
    fn rgb_components_named_colors() {
        assert_eq!(rgb_components(Color::Black), (0, 0, 0));
        assert_eq!(rgb_components(Color::White), (255, 255, 255));
    }
}

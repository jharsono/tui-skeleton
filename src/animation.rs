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
    /// Random braille dot patterns that change every frame — TV noise.
    /// Implies braille fill regardless of the `braille` flag.
    Noise,
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

/// Noise mode resting intensity — dim but visible.
const NOISE_INTENSITY: f32 = 0.3;

// ── Fill ────────────────────────────────────────────────────────────

/// Braille blank (U+2800). Adding 0..255 yields all dot patterns.
const BRAILLE_BASE: u32 = 0x2800;

const SOLID_FILL: char = '█';
const BRAILLE_FILL: char = '⣿'; // U+28FF — full braille block

/// Return the fill character for a cell.
///
/// - [`AnimationMode::Noise`]: random braille glyph per cell per frame
/// - `braille: true`: solid braille block (`⣿`)
/// - Otherwise: solid block (`█`)
pub(crate) fn cell_glyph(
    braille: bool,
    mode: AnimationMode,
    elapsed_ms: u64,
    row: u16,
    col: u16,
) -> char {
    if mode == AnimationMode::Noise {
        let h = cell_hash(elapsed_ms, row, col);
        return char::from_u32(BRAILLE_BASE + h as u32).unwrap_or(BRAILLE_FILL);
    }

    if braille { BRAILLE_FILL } else { SOLID_FILL }
}

/// Simple hash — enough entropy to look random, not cryptographic.
fn cell_hash(elapsed_ms: u64, row: u16, col: u16) -> u8 {
    let mut h = elapsed_ms
        .wrapping_mul(2654435761)
        .wrapping_add(row as u64 * 131)
        .wrapping_add(col as u64 * 65537);

    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1e995);
    h ^= h >> 15;

    h as u8
}

// ── Intensity ───────────────────────────────────────────────────────

/// Compute animation intensity for a single cell.
///
/// Returns a value in `[0.0, 1.0]` representing brightness progression
/// from `base` toward `highlight`.
pub(crate) fn cell_intensity(mode: AnimationMode, elapsed_ms: u64, col: u16, width: u16) -> f32 {
    match mode {
        AnimationMode::Sweep => sweep_intensity(elapsed_ms, col, width),
        AnimationMode::Breathe => breathe_intensity(elapsed_ms),
        AnimationMode::Plasma => plasma_intensity(elapsed_ms, col),
        AnimationMode::Noise => NOISE_INTENSITY,
    }
}

/// Returns true when the mode uses uniform (non-positional) intensity.
pub(crate) fn is_uniform(mode: AnimationMode) -> bool {
    matches!(mode, AnimationMode::Breathe | AnimationMode::Noise)
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
pub(crate) fn interpolate_color(
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
    fn noise_is_constant_intensity() {
        let a = cell_intensity(AnimationMode::Noise, 0, 0, 80);
        let b = cell_intensity(AnimationMode::Noise, 5000, 40, 80);
        assert_eq!(a, b);
        assert_eq!(a, NOISE_INTENSITY);
    }

    #[test]
    fn cell_glyph_solid_default() {
        assert_eq!(cell_glyph(false, AnimationMode::Breathe, 1000, 0, 0), '█');
        assert_eq!(cell_glyph(false, AnimationMode::Sweep, 1000, 0, 0), '█');
        assert_eq!(cell_glyph(false, AnimationMode::Plasma, 1000, 0, 0), '█');
    }

    #[test]
    fn cell_glyph_braille_fill() {
        assert_eq!(cell_glyph(true, AnimationMode::Breathe, 1000, 0, 0), '⣿');
        assert_eq!(cell_glyph(true, AnimationMode::Sweep, 1000, 0, 0), '⣿');
        assert_eq!(cell_glyph(true, AnimationMode::Plasma, 1000, 0, 0), '⣿');
    }

    #[test]
    fn cell_glyph_noise_is_random_braille() {
        let ch = cell_glyph(false, AnimationMode::Noise, 1000, 0, 0);
        assert!((0x2800..=0x28FF).contains(&(ch as u32)));
    }

    #[test]
    fn is_uniform_modes() {
        assert!(is_uniform(AnimationMode::Breathe));
        assert!(is_uniform(AnimationMode::Noise));
        assert!(!is_uniform(AnimationMode::Sweep));
        assert!(!is_uniform(AnimationMode::Plasma));
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

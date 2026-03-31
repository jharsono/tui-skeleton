# tui-skeleton — Animated Skeleton Loading Widgets

## Architecture

Stateless widgets driven by `elapsed_ms`. Animation state is computed purely from the timestamp — no mutable state, no tick tracking.

### Two Orthogonal Axes

Every widget exposes two independent dimensions:

1. **Animation mode** (`AnimationMode`) — how color changes over time
2. **Fill variant** (`braille(bool)`) — what character fills cells

| Mode | Color behavior | Glyph |
|---|---|---|
| **Breathe** (default) | 5s uniform sine pulse | `█` or `⣿` |
| **Sweep** | 800ms traveling cosine + 2s rest | `█` or `⣿` |
| **Plasma** | dual sine-wave interference, 2× contrast | `█` or `⣿` |
| **Noise** | constant dim intensity | random braille per cell per frame |

`braille(true)` renders `⣿` (solid braille block). `Noise` mode forces random braille regardless of the flag.

7 display combinations per widget: Breathe, Sweep, Plasma, Noise, Braille Breathe, Braille Sweep, Braille Plasma.

### Animation System (`animation.rs`, pub(crate))

- `cell_intensity(mode, elapsed_ms, col, width) -> f32` — per-cell brightness
- `cell_glyph(braille, mode, elapsed_ms, row, col) -> char` — fill character selection
- `is_uniform(mode) -> bool` — true for Breathe and Noise (hoistable)
- `interpolate_color(base, highlight, mode, intensity) -> Color` — RGB interpolation
- `cell_hash(elapsed_ms, row, col) -> u8` — deterministic noise pattern

### Widget Shapes

All widgets share: `new(elapsed_ms)`, builders (`mode`, `braille`, `base`, `highlight`, `block`), `Widget` impl. `#[must_use]` on structs.

| Widget | Shape-specific builders | Rendering |
|---|---|---|
| `SkeletonBlock` | — | Fills every cell via `render_skeleton_cells()` |
| `SkeletonTable` | `columns`, `rows`, `cell_widths`, `zebra` | Column separators, ragged per-cell fill, zebra |
| `SkeletonList` | `items`, `widths` | 1-row items + 1-row gaps, short widths |
| `SkeletonText` | `line_widths` | Paragraph with varying line widths |
| `SkeletonStreamingText` | `lines`, `duration_ms`, `repeat`, `line_widths` | Typewriter fill L→R T→B |
| `SkeletonBarChart` | `bars`, `bar_width`, `heights` | Vertical bars from bottom |
| `SkeletonHBarChart` | `bars`, `bar_height`, `widths` | Horizontal bars from left |
| `SkeletonBrailleBar` | `bars`, `fills`, `peak`, `peak_color`, `empty` | Braille gauge bars (`⢾⣿⡷`), Noise replaces structural glyphs |
| `SkeletonKvTable` | `pairs`, `key_width`, `value_widths` | Fixed key + `│` + variable value |
| `SkeletonLineChart` | `lines`, `filled` | Braille traces + fill area, wave drift |

`render_skeleton_cells()` in `block.rs` is the shared engine (pub(crate)). Takes `visible(row, col, width) -> bool` predicate. Used by Block, List, Text, StreamingText.

### Re-exports

`Color`, `Constraint`, `Block` re-exported so consumers avoid direct ratatui-core/ratatui-widgets deps.

## Pantry Integration

`pantry` feature gates `tui-pantry` 0.3.0 + `ratatui` as optional deps. Ingredient modules: `#[cfg(feature = "pantry")]` submodules via `#[path = "*.ingredient.rs"]`.

### Ingredient Structure

All ingredient files use `VARIANTS: &[(AnimationMode, bool, &str)]` — 7 entries. Each widget variant struct has `epoch`, `mode`, `braille`, `variant` fields. Block has a Compare variant showing all 7 stacked.

Chart/table ingredients render composite widgets with known chrome (headers, labels) always visible — skeleton fills only the data region.

### Panes (Use Cases)

5 pane types × 7 mode variants = 35 entries under the "Panes" tab. Each cycles skeleton↔content on a 10s loop (5s per phase). Known chrome (headers, labels, keys) always visible during skeleton phase.

| Pane | Known chrome | Skeleton data |
|---|---|---|
| Data Table | Header row (Node, Region, CPU, Status) | Row data |
| Article | Title line | Body text |
| Sidebar | Section header (Navigation) | Menu items |
| Dashboard | Bar labels, KV keys, table header | Bars, values, rows |
| Gauges | Metric labels (CPU, Memory, Disk, Network) | Braille bar per gauge |

Run: `cargo run --example widget_preview --features pantry`

### Demo

Interactive grid of all 10 widgets. `[m]` cycles modes (Breathe/Sweep/Plasma/Noise), `[b]` toggles braille fill, `[q]` quits.

Run: `cargo run --example demo`

## Current Focus

New widget variants landed: `SkeletonBrailleBar` (gauge shape) and `AnimationMode::Noise` (random braille per frame). Fill axis (`braille(bool)`) added to all widgets. `SkeletonTable` gained `cell_widths` for ragged per-cell fill. All pane ingredients render known chrome (headers/labels) outside the skeleton.

Not yet verified: tui-pantry standalone sync (`push`), Nix build after dependency additions, visual parity via `cargo pantry dump`.

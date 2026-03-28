# tui-skeleton — Animated Skeleton Loading Widgets

## Architecture

Stateless widgets driven by `elapsed_ms`. Animation state is computed purely from the timestamp — no mutable state, no tick tracking.

### Animation System

`animation.rs` (pub(crate)) — three modes sharing `cell_intensity(mode, elapsed_ms, col, width) -> f32`:

- **Breathe** (default) — uniform 5s sine pulse, hoisted outside per-cell loops
- **Sweep** — 800ms traveling cosine highlight + 2s rest
- **Plasma** — dual sine-wave interference, 2× contrast extrapolation

`interpolate_color(base, highlight, mode, intensity)` maps intensity to RGB. `rgb_components()` handles named Color variants.

### Widget Shapes

All 9 widgets share: `new(elapsed_ms)`, builder methods (`mode`, `base`, `highlight`, `block`), `Widget` impl. `#[must_use]` on structs only (builder methods inherit via return type).

| Widget | Key builder | Rendering |
|---|---|---|
| `SkeletonBlock` | — | Fills every cell via `render_skeleton_cells()` |
| `SkeletonTable` | `columns(&[Constraint])`, `rows`, `zebra` | Column separators (`│`), zebra offset +0.15 |
| `SkeletonList` | `items`, `widths` | 1-row items + 1-row gaps, short widths (30-55%) |
| `SkeletonText` | `line_widths` | Paragraph simulation, default [1.0, 1.0, 0.8, 1.0, 0.6] |
| `SkeletonStreamingText` | `lines`, `duration_ms`, `repeat`, `line_widths` | Typewriter fill L→R T→B, capped at 95% width |
| `SkeletonBarChart` | `bars`, `bar_width`, `heights` | Vertical bars from bottom, 1-cell gaps |
| `SkeletonHBarChart` | `bars`, `bar_height`, `widths` | Horizontal bars from left, 1-row gaps |
| `SkeletonKvTable` | `pairs`, `key_width`, `value_widths` | Fixed key + dim `│` + variable value |
| `SkeletonLineChart` | `lines`, `filled` | Braille traces + `█` fill area, wave drift |

`render_skeleton_cells()` in `block.rs` is the shared engine (pub(crate)). Takes a `visible(row, col, width) -> bool` predicate. Used by Block, List, Text, StreamingText.

### Re-exports

`Color`, `Constraint`, `Block` are re-exported so consumers don't need direct ratatui-core/ratatui-widgets dependencies.

## Pantry Integration

`pantry` feature gates `tui-pantry` 0.3.0 + `ratatui` as optional deps. Ingredient modules are `#[cfg(feature = "pantry")]` submodules of each widget file via `#[path = "*.ingredient.rs"]`.

Widget modules are `pub` (required for pantry macro access). Internal functions remain `pub(crate)`.

### Ingredient Structure

All ingredient files use a single parameterized struct with `mode` + `variant` fields, registered via a `VARIANTS` const. Each has 3 mode variants (Breathe/Sweep/Plasma) with `PropInfo`. Block additionally has a Compare variant.

### Panes (Use Cases)

4 pane ingredients × 3 mode variants = 12 entries under the "Panes" tab. Each cycles skeleton↔content on a 10s loop (5s per phase). Content is constrained to match skeleton dimensions.

| Pane | Skeleton widget | Loaded content |
|---|---|---|
| Data Table | `SkeletonTable` 5×4 | Node status table with header |
| Article | `SkeletonText` 5 lines, 52 col cap | Lorem ipsum paragraph |
| Sidebar | `SkeletonList` 5 items | Navigation menu |
| Dashboard | `SkeletonBarChart` + `SkeletonKvTable` + `SkeletonTable` | Bars + KV pairs + table |

Run: `cargo run --example widget_preview --features pantry`.

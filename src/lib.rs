//! Animated skeleton loading widgets for [Ratatui](https://ratatui.rs).
//!
//! Provides placeholder widgets that pulse, sweep, or shimmer while data loads.
//! All widgets are stateless — pass `elapsed_ms` from your event loop and the
//! animation state is computed purely from the timestamp.
//!
//! # Widgets
//!
//! - [`SkeletonBlock`] — Solid filled rectangle (atomic unit)
//! - [`SkeletonTable`] — Rows with column separators and zebra striping
//! - [`SkeletonList`] — Short spaced items with ragged edges (menu/sidebar)
//! - [`SkeletonText`] — Paragraph simulation with varying line widths
//! - [`SkeletonBarChart`] — Vertical bars of varying height
//! - [`SkeletonHBarChart`] — Horizontal bars of varying length
//! - [`SkeletonKvTable`] — Key-value pairs (properties/detail panel)
//! - [`SkeletonLineChart`] — Braille line chart with overlapping wave traces
//!
//! # Example
//!
//! ```rust
//! use ratatui_core::style::Color;
//! use tui_skeleton::{SkeletonBlock, AnimationMode};
//!
//! let elapsed_ms = 1000u64;
//! let widget = SkeletonBlock::new(elapsed_ms)
//!     .mode(AnimationMode::Breathe)
//!     .base(Color::Rgb(30, 22, 58))
//!     .highlight(Color::Rgb(49, 40, 78));
//! ```
//!
//! # Adaptive Tick Rate
//!
//! Skeleton animations look best at ~20 FPS ([`TICK_ANIMATED`]) but most TUI
//! applications tick at ~5 FPS ([`TICK_IDLE`]) for static content. The
//! recommended pattern:
//!
//! 1. Track whether any skeleton widget is currently visible
//! 2. Use [`TICK_ANIMATED`] when skeletons are on screen
//! 3. Revert to [`TICK_IDLE`] when all data has loaded
//!
//! This keeps CPU usage low while delivering smooth animations during loading.

pub mod animation;
mod bar_chart;
mod block;
mod hbar_chart;
mod kv_table;
mod line_chart;
mod list;
mod table;
mod text;

pub use animation::AnimationMode;
pub use bar_chart::SkeletonBarChart;
pub use block::SkeletonBlock;
pub use hbar_chart::SkeletonHBarChart;
pub use kv_table::SkeletonKvTable;
pub use line_chart::SkeletonLineChart;
pub use list::SkeletonList;
pub use table::SkeletonTable;
pub use text::SkeletonText;

use std::time::Duration;

/// Recommended tick interval when skeleton widgets are visible (50ms / 20 FPS).
pub const TICK_ANIMATED: Duration = Duration::from_millis(50);

/// Recommended tick interval for static content (200ms / 5 FPS).
pub const TICK_IDLE: Duration = Duration::from_millis(200);

/// Default colors that work on both dark and light terminals.
pub mod defaults {
    use ratatui_core::style::Color;

    pub const BASE: Color = Color::DarkGray;
    pub const HIGHLIGHT: Color = Color::Gray;
}

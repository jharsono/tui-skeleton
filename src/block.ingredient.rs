use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};
use tui_pantry::{layout::render_centered, Ingredient, PropInfo};

use super::SkeletonBlock;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Solid filled rectangle with animated brightness — the atomic skeleton unit.";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "elapsed_ms",
        ty: "u64",
        description: "Timestamp driving the animation cycle",
    },
    PropInfo {
        name: "mode",
        ty: "AnimationMode",
        description:
            "Breathe (uniform pulse), Sweep (traveling highlight), Plasma (dual sine waves)",
    },
    PropInfo {
        name: "base",
        ty: "Color",
        description: "Dim resting color (default: DarkGray)",
    },
    PropInfo {
        name: "highlight",
        ty: "Color",
        description: "Peak brightness color (default: Gray)",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    let mut out: Vec<Box<dyn Ingredient>> = VARIANTS
        .iter()
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(BlockVariant {
                epoch: Instant::now(),
                mode,
                variant: name,
            })
        })
        .collect();

    out.push(Box::new(BlockCompare {
        epoch: Instant::now(),
    }));

    out
}

const VARIANTS: &[(AnimationMode, &str)] = &[
    (AnimationMode::Breathe, "Breathe (default)"),
    (AnimationMode::Sweep, "Sweep"),
    (AnimationMode::Plasma, "Plasma"),
];

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

// ── Mode variants ──

struct BlockVariant {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl Ingredient for BlockVariant {
    fn group(&self) -> &str {
        "Block"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::block"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            SkeletonBlock::new(elapsed_ms(self.epoch)).mode(self.mode),
            None,
            Some(Constraint::Length(3)),
            area,
            buf,
        );
    }
}

// ── Compare (all three stacked) ──

struct BlockCompare {
    epoch: Instant,
}

impl Ingredient for BlockCompare {
    fn group(&self) -> &str {
        "Block"
    }
    fn name(&self) -> &str {
        "Compare"
    }
    fn source(&self) -> &str {
        "tui_skeleton::block"
    }
    fn description(&self) -> &str {
        "All three animation modes stacked: Sweep, Breathe, Plasma."
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui_core::text::Line;

        let ms = elapsed_ms(self.epoch);

        let [_, sweep_label, sweep_area, gap_1, breathe_label, breathe_area, gap_2, plasma_label, plasma_area, _] =
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(area);

        Line::raw("Sweep").render(sweep_label, buf);
        SkeletonBlock::new(ms)
            .mode(AnimationMode::Sweep)
            .render(sweep_area, buf);

        let _ = gap_1;

        Line::raw("Breathe").render(breathe_label, buf);
        SkeletonBlock::new(ms).render(breathe_area, buf);

        let _ = gap_2;

        Line::raw("Plasma").render(plasma_label, buf);
        SkeletonBlock::new(ms)
            .mode(AnimationMode::Plasma)
            .render(plasma_area, buf);
    }
}

use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{layout::render_centered, Ingredient, PropInfo};

use super::SkeletonHBarChart;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Horizontal bars of varying length extending from the left, mimicking a ranking display.";

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
        name: "bars",
        ty: "u16",
        description: "Number of horizontal bars (default: 5)",
    },
    PropInfo {
        name: "bar_height",
        ty: "u16",
        description: "Height of each bar in rows (default: 1)",
    },
    PropInfo {
        name: "widths",
        ty: "&[f32]",
        description: "Per-bar width fractions, cycling",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(HBarChartVariant {
                epoch: Instant::now(),
                mode,
                variant: name,
            })
        })
        .collect()
}

const VARIANTS: &[(AnimationMode, &str)] = &[
    (AnimationMode::Breathe, "Breathe (default)"),
    (AnimationMode::Sweep, "Sweep"),
    (AnimationMode::Plasma, "Plasma"),
];

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct HBarChartVariant {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl Ingredient for HBarChartVariant {
    fn group(&self) -> &str {
        "Horizontal Bar Chart"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::hbar_chart"
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
            SkeletonHBarChart::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .bars(5),
            None,
            Some(Constraint::Length(9)),
            area,
            buf,
        );
    }
}

use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{layout::render_centered, Ingredient, PropInfo};

use super::SkeletonBarChart;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Vertical bars of varying height rising from the bottom, mimicking a bar chart.";

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
        description: "Number of vertical bars (default: 6)",
    },
    PropInfo {
        name: "bar_width",
        ty: "u16",
        description: "Width of each bar in cells (default: 3)",
    },
    PropInfo {
        name: "heights",
        ty: "&[f32]",
        description: "Per-bar height fractions, cycling",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(BarChartVariant {
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

struct BarChartVariant {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl Ingredient for BarChartVariant {
    fn group(&self) -> &str {
        "Bar Chart"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::bar_chart"
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
            SkeletonBarChart::new(elapsed_ms(self.epoch)).mode(self.mode),
            None,
            Some(Constraint::Length(10)),
            area,
            buf,
        );
    }
}

use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

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
        description: "Breathe (uniform pulse), Sweep (traveling highlight), Plasma (dual sine waves), Noise (TV noise)",
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
        .map(|&(mode, braille, name)| -> Box<dyn Ingredient> {
            Box::new(BarChartVariant {
                epoch: Instant::now(),
                mode,
                braille,
                variant: name,
            })
        })
        .collect()
}

const VARIANTS: &[(AnimationMode, bool, &str)] = &[
    (AnimationMode::Breathe, false, "Breathe (default)"),
    (AnimationMode::Sweep, false, "Sweep"),
    (AnimationMode::Plasma, false, "Plasma"),
    (AnimationMode::Noise, false, "Noise"),
    (AnimationMode::Breathe, true, "Braille Breathe"),
    (AnimationMode::Sweep, true, "Braille Sweep"),
    (AnimationMode::Plasma, true, "Braille Plasma"),
];

const BAR_LABELS: [&str; 6] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct BarChartVariant {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
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
            BarChartWithLabels {
                ms: elapsed_ms(self.epoch),
                mode: self.mode,
                braille: self.braille,
            },
            None,
            Some(Constraint::Length(10)),
            area,
            buf,
        );
    }
}

/// Composite widget: skeleton bar chart with x-axis labels below.
struct BarChartWithLabels {
    ms: u64,
    mode: AnimationMode,
    braille: bool,
}

impl Widget for BarChartWithLabels {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [chart_area, label_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        SkeletonBarChart::new(self.ms)
            .mode(self.mode)
            .braille(self.braille)
            .render(chart_area, buf);

        // Render x-axis labels aligned to bar positions.
        let bar_width = 3u16;
        let stride = bar_width + 1;

        for (i, &label) in BAR_LABELS.iter().enumerate() {
            let x = label_area.x + (i as u16) * stride;

            if x + bar_width > label_area.right() {
                break;
            }

            Line::from(label).render(Rect::new(x, label_area.y, bar_width, 1), buf);
        }
    }
}

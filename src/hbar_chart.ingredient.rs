use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    text::Line,
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

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
        description: "Breathe (uniform pulse), Sweep (traveling highlight), Plasma (dual sine waves), Noise (TV noise)",
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
        .map(|&(mode, braille, name)| -> Box<dyn Ingredient> {
            Box::new(HBarChartVariant {
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

const ROW_LABELS: [&str; 5] = ["Alpha", "Bravo", "Charlie", "Delta", "Echo"];
const LABEL_WIDTH: u16 = 8;

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct HBarChartVariant {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
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
            HBarChartWithLabels {
                ms: elapsed_ms(self.epoch),
                mode: self.mode,
                braille: self.braille,
            },
            None,
            Some(Constraint::Length(9)),
            area,
            buf,
        );
    }
}

/// Composite widget: row labels + skeleton horizontal bar chart.
struct HBarChartWithLabels {
    ms: u64,
    mode: AnimationMode,
    braille: bool,
}

impl Widget for HBarChartWithLabels {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bar_x = area.x + LABEL_WIDTH;
        let bar_width = area.width.saturating_sub(LABEL_WIDTH);

        // Row labels on the left, aligned to bar stride (bar_height=1, gap=1 → stride=2).
        for (i, &label) in ROW_LABELS.iter().enumerate() {
            let y = area.y + (i as u16) * 2;

            if y >= area.bottom() {
                break;
            }

            Line::from(label)
                .style(Style::new().bold())
                .render(Rect::new(area.x, y, LABEL_WIDTH, 1), buf);
        }

        if bar_width > 0 {
            SkeletonHBarChart::new(self.ms)
                .mode(self.mode)
                .braille(self.braille)
                .bars(5)
                .render(Rect::new(bar_x, area.y, bar_width, area.height), buf);
        }
    }
}

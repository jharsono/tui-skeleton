use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

use super::SkeletonText;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Paragraph simulation with lines of varying width, mimicking loading text content.";

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
        name: "line_widths",
        ty: "&[f32]",
        description: "Per-line width fractions, cycling (default: [1.0, 1.0, 0.8, 1.0, 0.6])",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, braille, name)| -> Box<dyn Ingredient> {
            Box::new(TextVariant {
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

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct TextVariant {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl Ingredient for TextVariant {
    fn group(&self) -> &str {
        "Text"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::text"
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
            SkeletonText::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .braille(self.braille),
            None,
            Some(Constraint::Length(5)),
            area,
            buf,
        );
    }
}

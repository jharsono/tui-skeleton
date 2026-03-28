use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{layout::render_centered, Ingredient, PropInfo};

use super::SkeletonStreamingText;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Typewriter-style text fill simulating streaming chat output with ragged paragraph shape.";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "elapsed_ms",
        ty: "u64",
        description: "Timestamp driving both fill progress and animation cycle",
    },
    PropInfo {
        name: "mode",
        ty: "AnimationMode",
        description:
            "Breathe (uniform pulse), Sweep (traveling highlight), Plasma (dual sine waves)",
    },
    PropInfo {
        name: "lines",
        ty: "u16",
        description: "Total lines of text to fill (default: 5)",
    },
    PropInfo {
        name: "duration_ms",
        ty: "u64",
        description: "Milliseconds to complete the typewriter fill (default: 3000)",
    },
    PropInfo {
        name: "repeat",
        ty: "bool",
        description:
            "Loop the fill cycle; when false, fills once and keeps pulsing (default: false)",
    },
    PropInfo {
        name: "line_widths",
        ty: "&[f32]",
        description: "Per-line width fractions, cycling",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(StreamingTextVariant {
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

struct StreamingTextVariant {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl Ingredient for StreamingTextVariant {
    fn group(&self) -> &str {
        "Streaming Text"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::streaming_text"
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
            SkeletonStreamingText::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .repeat(true),
            None,
            Some(Constraint::Length(5)),
            area,
            buf,
        );
    }
}

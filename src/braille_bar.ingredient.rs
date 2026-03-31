use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

use super::SkeletonBrailleBar;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Braille progress bars with rounded caps and optional peak marker, stacked vertically.";

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
        description: "Number of stacked bars (default: 3)",
    },
    PropInfo {
        name: "fills",
        ty: "&[f32]",
        description: "Per-bar fill fractions, cycling",
    },
    PropInfo {
        name: "peak",
        ty: "f32",
        description: "Optional peak marker position as fraction (0.0..=1.0)",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    let mut out: Vec<Box<dyn Ingredient>> = VARIANTS
        .iter()
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(BrailleBarVariant {
                epoch: Instant::now(),
                mode,
                variant: name,
                with_peak: false,
            })
        })
        .collect();

    out.push(Box::new(BrailleBarVariant {
        epoch: Instant::now(),
        mode: AnimationMode::Breathe,
        variant: "With Peak Marker",
        with_peak: true,
    }));

    out
}

const VARIANTS: &[(AnimationMode, &str)] = &[
    (AnimationMode::Breathe, "Breathe (default)"),
    (AnimationMode::Sweep, "Sweep"),
    (AnimationMode::Plasma, "Plasma"),
    (AnimationMode::Noise, "Noise"),
];

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct BrailleBarVariant {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
    with_peak: bool,
}

impl Ingredient for BrailleBarVariant {
    fn group(&self) -> &str {
        "Braille Bar"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::braille_bar"
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
        let mut widget = SkeletonBrailleBar::new(elapsed_ms(self.epoch))
            .mode(self.mode)
            .bars(4);

        if self.with_peak {
            widget = widget
                .peak(0.78)
                .peak_color(ratatui_core::style::Color::Rgb(251, 146, 60));
        }

        render_centered(widget, None, Some(Constraint::Length(7)), area, buf);
    }
}

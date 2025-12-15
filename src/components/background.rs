//! Background component with customizable patterns.

use dioxus::prelude::*;

/// Background pattern variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundVariant {
    #[default]
    Dots,
    Lines,
    Cross,
}

/// Background component props.
#[derive(Props, Clone, PartialEq)]
pub struct BackgroundProps {
    /// Background pattern variant.
    #[props(default)]
    pub variant: BackgroundVariant,
    /// Gap between pattern elements.
    #[props(default = 20.0)]
    pub gap: f64,
    /// Size of the pattern elements.
    #[props(default = 1.0)]
    pub size: f64,
    /// Color of the pattern.
    #[props(default = "#ddd".to_string())]
    pub color: String,
    /// Background color.
    #[props(default = "#f8f8f8".to_string())]
    pub background_color: String,
}

/// Background component for the flow.
#[component]
pub fn Background(props: BackgroundProps) -> Element {
    let pattern_id = "dioxus-flow-background-pattern";

    let pattern_content = match props.variant {
        BackgroundVariant::Dots => {
            rsx! {
                circle {
                    cx: "{props.gap / 2.0}",
                    cy: "{props.gap / 2.0}",
                    r: "{props.size}",
                    fill: "{props.color}",
                }
            }
        }
        BackgroundVariant::Lines => {
            rsx! {
                line {
                    x1: "0",
                    y1: "0",
                    x2: "{props.gap}",
                    y2: "0",
                    stroke: "{props.color}",
                    stroke_width: "{props.size}",
                }
            }
        }
        BackgroundVariant::Cross => {
            rsx! {
                line {
                    x1: "0",
                    y1: "{props.gap / 2.0}",
                    x2: "{props.gap}",
                    y2: "{props.gap / 2.0}",
                    stroke: "{props.color}",
                    stroke_width: "{props.size}",
                }
                line {
                    x1: "{props.gap / 2.0}",
                    y1: "0",
                    x2: "{props.gap / 2.0}",
                    y2: "{props.gap}",
                    stroke: "{props.color}",
                    stroke_width: "{props.size}",
                }
            }
        }
    };

    rsx! {
        svg {
            class: "dioxus-flow-background",
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 0;",

            defs {
                pattern {
                    id: "{pattern_id}",
                    width: "{props.gap}",
                    height: "{props.gap}",
                    pattern_units: "userSpaceOnUse",
                    {pattern_content}
                }
            }

            rect {
                width: "100%",
                height: "100%",
                fill: "{props.background_color}",
            }
            rect {
                width: "100%",
                height: "100%",
                fill: "url(#{pattern_id})",
            }
        }
    }
}

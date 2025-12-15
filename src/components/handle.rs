//! Handle component for connection points on nodes.

use crate::types::HandlePosition;
use dioxus::prelude::*;

/// Handle type - source (output) or target (input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HandleType {
    #[default]
    Source,
    Target,
}

/// Handle component props.
#[derive(Props, Clone, PartialEq)]
pub struct HandleProps {
    /// Handle type (source or target).
    #[props(default)]
    pub handle_type: HandleType,
    /// Handle position on the node.
    #[props(default)]
    pub position: HandlePosition,
    /// Whether the handle is connectable.
    #[props(default = true)]
    pub connectable: bool,
    /// Custom ID for the handle.
    #[props(default)]
    pub id: Option<String>,
    /// Additional CSS class.
    #[props(default)]
    pub class: String,
    /// Callback when connection starts from this handle.
    #[props(default)]
    pub on_connect_start: Option<EventHandler<HandlePosition>>,
    /// Callback when connection ends at this handle.
    #[props(default)]
    pub on_connect_end: Option<EventHandler<HandlePosition>>,
}

/// Handle component for connection points on nodes.
#[component]
pub fn Handle(props: HandleProps) -> Element {
    let position_class = match props.position {
        HandlePosition::Top => "dioxus-flow-handle-top",
        HandlePosition::Right => "dioxus-flow-handle-right",
        HandlePosition::Bottom => "dioxus-flow-handle-bottom",
        HandlePosition::Left => "dioxus-flow-handle-left",
    };

    let type_class = match props.handle_type {
        HandleType::Source => "dioxus-flow-handle-source",
        HandleType::Target => "dioxus-flow-handle-target",
    };

    let position = props.position;
    let on_connect_start = props.on_connect_start.clone();
    let on_connect_end = props.on_connect_end.clone();

    rsx! {
        div {
            class: "dioxus-flow-handle {position_class} {type_class} {props.class}",
            "data-handle-type": if props.handle_type == HandleType::Source { "source" } else { "target" },
            "data-handle-position": match props.position {
                HandlePosition::Top => "top",
                HandlePosition::Right => "right",
                HandlePosition::Bottom => "bottom",
                HandlePosition::Left => "left",
            },
            onmousedown: move |evt| {
                if props.handle_type == HandleType::Source {
                    evt.stop_propagation();
                    if let Some(handler) = &on_connect_start {
                        handler.call(position);
                    }
                }
            },
            onmouseup: move |evt| {
                if props.handle_type == HandleType::Target {
                    evt.stop_propagation();
                    if let Some(handler) = &on_connect_end {
                        handler.call(position);
                    }
                }
            },
        }
    }
}

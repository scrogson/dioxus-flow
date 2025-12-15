//! Edge component for connections between nodes.

use crate::types::{Edge, EdgeId, EdgeType, HandlePosition, Position};
use crate::utils::get_edge_path;
use dioxus::prelude::*;

/// Edge component props.
#[derive(Props, Clone, PartialEq)]
pub struct EdgeComponentProps {
    /// The edge data.
    pub edge: Edge,
    /// Source position in flow coordinates.
    pub source_position: Position,
    /// Target position in flow coordinates.
    pub target_position: Position,
    /// Callback when edge is selected.
    #[props(default)]
    pub on_select: Option<EventHandler<EdgeId>>,
}

/// Edge component for rendering connections.
#[component]
pub fn EdgeComponent(props: EdgeComponentProps) -> Element {
    let edge = &props.edge;
    let path = get_edge_path(
        edge.edge_type,
        props.source_position,
        props.target_position,
        edge.source_handle,
        edge.target_handle,
    );

    let selected_class = if edge.selected {
        "dioxus-flow-edge-selected"
    } else {
        ""
    };
    let animated_class = if edge.animated {
        "dioxus-flow-edge-animated"
    } else {
        ""
    };

    let edge_type_class = match edge.edge_type {
        EdgeType::Bezier => "dioxus-flow-edge-bezier",
        EdgeType::Straight => "dioxus-flow-edge-straight",
        EdgeType::Step => "dioxus-flow-edge-step",
        EdgeType::SmoothStep => "dioxus-flow-edge-smoothstep",
    };

    let on_select = props.on_select.clone();
    let edge_id = edge.id.clone();

    // Calculate label position (middle of the path)
    let label_x = (props.source_position.x + props.target_position.x) / 2.0;
    let label_y = (props.source_position.y + props.target_position.y) / 2.0;

    rsx! {
        g {
            class: "dioxus-flow-edge {edge_type_class} {selected_class} {animated_class} {edge.class}",
            "data-id": "{edge.id}",
            // Invisible wider path for easier selection
            path {
                class: "dioxus-flow-edge-interaction",
                d: "{path}",
                fill: "none",
                stroke: "transparent",
                stroke_width: "20",
                onclick: move |evt| {
                    evt.stop_propagation();
                    if let Some(handler) = &on_select {
                        handler.call(edge_id.clone());
                    }
                },
            }
            // Visible edge path
            path {
                class: "dioxus-flow-edge-path",
                d: "{path}",
                fill: "none",
                stroke: "{edge.stroke}",
                stroke_width: "{edge.stroke_width}",
                marker_end: "url(#dioxus-flow-arrowhead)",
            }
            // Edge label
            if let Some(label) = &edge.label {
                foreignObject {
                    x: "{label_x - 50.0}",
                    y: "{label_y - 10.0}",
                    width: "100",
                    height: "20",
                    class: "dioxus-flow-edge-label-container",
                    div {
                        class: "dioxus-flow-edge-label",
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Connection line component for when dragging to create a new connection.
#[derive(Props, Clone, PartialEq)]
pub struct ConnectionLineProps {
    /// Source position.
    pub source: Position,
    /// Source handle position.
    pub source_handle: HandlePosition,
    /// Target position (mouse position).
    pub target: Position,
    /// Edge type for the connection line.
    #[props(default)]
    pub edge_type: EdgeType,
}

#[component]
pub fn ConnectionLine(props: ConnectionLineProps) -> Element {
    let path = get_edge_path(
        props.edge_type,
        props.source,
        props.target,
        props.source_handle,
        HandlePosition::Top, // Default target handle
    );

    rsx! {
        g {
            class: "dioxus-flow-connection-line",
            path {
                d: "{path}",
                fill: "none",
                stroke: "#b1b1b7",
                stroke_width: "2",
                stroke_dasharray: "5,5",
            }
        }
    }
}

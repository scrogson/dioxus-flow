//! Node component for the flow.

use crate::types::{HandlePosition, Node, NodeId, Position};
use dioxus::prelude::*;

/// Node component props.
#[derive(Props, Clone, PartialEq)]
pub struct NodeComponentProps<T: Clone + PartialEq + 'static> {
    /// The node data.
    pub node: Node<T>,
    /// Current zoom level.
    pub zoom: f64,
    /// Whether the node is currently being dragged.
    #[props(default)]
    pub dragging: bool,
    /// Callback when node is selected.
    #[props(default)]
    pub on_select: Option<EventHandler<NodeId>>,
    /// Callback when node drag starts.
    #[props(default)]
    pub on_drag_start: Option<EventHandler<(NodeId, Position)>>,
    /// Callback when node is dragged.
    #[props(default)]
    pub on_drag: Option<EventHandler<(NodeId, Position)>>,
    /// Callback when node drag ends.
    #[props(default)]
    pub on_drag_end: Option<EventHandler<NodeId>>,
    /// Callback when connection starts from a handle.
    #[props(default)]
    pub on_connect_start: Option<EventHandler<(NodeId, HandlePosition)>>,
    /// Callback when connection ends at a handle.
    #[props(default)]
    pub on_connect_end: Option<EventHandler<(NodeId, HandlePosition)>>,
    /// Custom node renderer.
    #[props(default)]
    pub children: Element,
}

/// Default node component.
#[component]
pub fn NodeComponent<T: Clone + PartialEq + 'static>(props: NodeComponentProps<T>) -> Element {
    let node = &props.node;
    let node_id = node.id.clone();

    let selected_class = if node.selected {
        "dioxus-flow-node-selected"
    } else {
        ""
    };
    let dragging_class = if props.dragging {
        "dioxus-flow-node-dragging"
    } else {
        ""
    };

    let style = format!(
        "position: absolute; left: {}px; top: {}px; transform: translate(0, 0);{}",
        node.position.x,
        node.position.y,
        node.style
            .iter()
            .map(|(k, v)| format!(" {}: {};", k, v))
            .collect::<String>()
    );

    let on_select = props.on_select.clone();
    let on_drag_start = props.on_drag_start.clone();
    let on_connect_start = props.on_connect_start.clone();
    let on_connect_end = props.on_connect_end.clone();
    let draggable = node.draggable;
    let connectable = node.connectable;

    rsx! {
        div {
            class: "dioxus-flow-node dioxus-flow-node-{node.node_type} {selected_class} {dragging_class} {node.class}",
            style: "{style}",
            "data-id": "{node.id}",
            onclick: {
                let node_id = node_id.clone();
                move |evt: MouseEvent| {
                    evt.stop_propagation();
                    if let Some(handler) = &on_select {
                        handler.call(node_id.clone());
                    }
                }
            },
            onmousedown: {
                let node_id = node_id.clone();
                move |evt: MouseEvent| {
                    if draggable {
                        evt.stop_propagation();
                        if let Some(handler) = &on_drag_start {
                            let coords = evt.client_coordinates();
                            handler.call((node_id.clone(), Position::new(coords.x, coords.y)));
                        }
                    }
                }
            },
            // Render handles if connectable
            if connectable {
                // Source handle (bottom)
                div {
                    class: "dioxus-flow-handle dioxus-flow-handle-bottom dioxus-flow-handle-source",
                    "data-handle-type": "source",
                    "data-handle-position": "bottom",
                    onmousedown: {
                        let node_id = node_id.clone();
                        move |evt: MouseEvent| {
                            evt.stop_propagation();
                            if let Some(handler) = &on_connect_start {
                                handler.call((node_id.clone(), HandlePosition::Bottom));
                            }
                        }
                    },
                }
                // Target handle (top)
                div {
                    class: "dioxus-flow-handle dioxus-flow-handle-top dioxus-flow-handle-target",
                    "data-handle-type": "target",
                    "data-handle-position": "top",
                    onmouseup: {
                        let node_id = node_id.clone();
                        move |evt: MouseEvent| {
                            evt.stop_propagation();
                            if let Some(handler) = &on_connect_end {
                                handler.call((node_id.clone(), HandlePosition::Top));
                            }
                        }
                    },
                }
            }
            // Node content
            if props.children.is_ok() {
                {props.children}
            } else {
                // Default node content - just show the type
                div {
                    class: "dioxus-flow-node-content",
                    "{node.node_type}"
                }
            }
        }
    }
}

//! Node component for the flow.

use crate::types::{HandleKind, HandlePosition, Node, NodeId, Position};
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

    // Build style with explicit dimensions if set
    let dimensions = match (node.width, node.height) {
        (Some(w), Some(h)) => format!(" width: {}px; height: {}px;", w, h),
        (Some(w), None) => format!(" width: {}px;", w),
        (None, Some(h)) => format!(" height: {}px;", h),
        (None, None) => String::new(),
    };

    let style = format!(
        "position: absolute; left: {}px; top: {}px; box-sizing: border-box; pointer-events: auto;{}{}",
        node.position.x,
        node.position.y,
        dimensions,
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
            // Render handles from node.handles
            if connectable && !node.handles.is_empty() {
                for handle in node.handles.iter() {
                    {
                        let node_width = node.width.unwrap_or(150.0);
                        let node_height = node.height.unwrap_or(40.0);
                        let handle_id = handle.id.clone();
                        let handle_pos = handle.position;
                        let handle_kind = handle.kind;

                        // Calculate position - use percentage for centering, pixels for explicit offsets
                        let (style_pos, pos_class) = match handle_pos {
                            HandlePosition::Top => {
                                if let Some(offset) = handle.offset {
                                    let x = offset * node_width;
                                    (format!("top: 0; left: {}px; transform: translate(-50%, -50%);", x), "top")
                                } else {
                                    ("top: 0; left: 50%; transform: translate(-50%, -50%);".to_string(), "top")
                                }
                            }
                            HandlePosition::Bottom => {
                                if let Some(offset) = handle.offset {
                                    let x = offset * node_width;
                                    (format!("bottom: 0; left: {}px; transform: translate(-50%, 50%);", x), "bottom")
                                } else {
                                    ("bottom: 0; left: 50%; transform: translate(-50%, 50%);".to_string(), "bottom")
                                }
                            }
                            HandlePosition::Left => {
                                if let Some(offset) = handle.offset {
                                    let y = offset * node_height;
                                    (format!("left: 0; top: {}px; transform: translate(-50%, -50%);", y), "left")
                                } else {
                                    ("left: 0; top: 50%; transform: translate(-50%, -50%);".to_string(), "left")
                                }
                            }
                            HandlePosition::Right => {
                                if let Some(offset) = handle.offset {
                                    let y = offset * node_height;
                                    (format!("right: 0; top: {}px; transform: translate(50%, -50%);", y), "right")
                                } else {
                                    ("right: 0; top: 50%; transform: translate(50%, -50%);".to_string(), "right")
                                }
                            }
                        };

                        let kind_class = match handle_kind {
                            HandleKind::Source => "source",
                            HandleKind::Target => "target",
                        };

                        rsx! {
                            div {
                                key: "{handle_id}",
                                class: "dioxus-flow-handle dioxus-flow-handle-{pos_class} dioxus-flow-handle-{kind_class}",
                                style: "position: absolute; {style_pos}",
                                "data-handle-id": "{handle_id}",
                                "data-handle-type": "{kind_class}",
                                "data-handle-position": "{pos_class}",
                                onmousedown: {
                                    let node_id = node_id.clone();
                                    let handle_pos = handle_pos;
                                    move |evt: MouseEvent| {
                                        if handle_kind == HandleKind::Source {
                                            evt.stop_propagation();
                                            if let Some(handler) = &on_connect_start {
                                                handler.call((node_id.clone(), handle_pos));
                                            }
                                        }
                                    }
                                },
                                onmouseup: {
                                    let node_id = node_id.clone();
                                    let handle_pos = handle_pos;
                                    move |evt: MouseEvent| {
                                        if handle_kind == HandleKind::Target {
                                            evt.stop_propagation();
                                            if let Some(handler) = &on_connect_end {
                                                handler.call((node_id.clone(), handle_pos));
                                            }
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            } else if connectable {
                // Fallback: render default top/bottom handles
                div {
                    class: "dioxus-flow-handle dioxus-flow-handle-bottom dioxus-flow-handle-source",
                    style: "position: absolute; bottom: 0; left: 50%; transform: translate(-50%, 50%);",
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
                div {
                    class: "dioxus-flow-handle dioxus-flow-handle-top dioxus-flow-handle-target",
                    style: "position: absolute; top: 0; left: 50%; transform: translate(-50%, -50%);",
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
            // Node content - show label, falling back to id
            div {
                class: "dioxus-flow-node-content",
                {node.label.as_ref().unwrap_or(&node.id).clone()}
            }
        }
    }
}

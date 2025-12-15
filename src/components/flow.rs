//! Main Flow component.

use crate::components::edge::{ConnectionLine, EdgeComponent};
use crate::components::node::NodeComponent;
use crate::hooks::FlowState;
use crate::types::{Edge, HandlePosition, NodeId, Position, Viewport};
use dioxus::prelude::*;
use dioxus::html::geometry::WheelDelta;

/// Flow component props.
#[derive(Props, Clone, PartialEq)]
pub struct FlowProps<T: Clone + PartialEq + 'static> {
    /// Signal containing the flow state.
    pub state: Signal<FlowState<T>>,
    /// Minimum zoom level.
    #[props(default = 0.1)]
    pub min_zoom: f64,
    /// Maximum zoom level.
    #[props(default = 4.0)]
    pub max_zoom: f64,
    /// Whether panning is enabled.
    #[props(default = true)]
    pub pan_on_drag: bool,
    /// Whether zooming is enabled.
    #[props(default = true)]
    pub zoom_on_scroll: bool,
    /// Whether nodes are selectable.
    #[props(default = true)]
    pub nodes_selectable: bool,
    /// Whether edges are selectable.
    #[props(default = true)]
    pub edges_selectable: bool,
    /// Whether nodes are draggable.
    #[props(default = true)]
    pub nodes_draggable: bool,
    /// Whether nodes are connectable.
    #[props(default = true)]
    pub nodes_connectable: bool,
    /// Callback for node click.
    #[props(default)]
    pub on_node_click: Option<EventHandler<NodeId>>,
    /// Callback for edge click.
    #[props(default)]
    pub on_edge_click: Option<EventHandler<String>>,
    /// Callback for pane click.
    #[props(default)]
    pub on_pane_click: Option<EventHandler<Position>>,
    /// Callback for viewport change.
    #[props(default)]
    pub on_viewport_change: Option<EventHandler<Viewport>>,
    /// Callback when a new connection is made.
    #[props(default)]
    pub on_connect: Option<EventHandler<Edge>>,
    /// Callback when node position changes.
    #[props(default)]
    pub on_node_drag: Option<EventHandler<(NodeId, Position)>>,
    /// Additional CSS class for the container.
    #[props(default)]
    pub class: String,
    /// Additional children to render inside the flow.
    #[props(default)]
    pub children: Element,
}

/// Main Flow component.
#[component]
pub fn Flow<T: Clone + Default + PartialEq + 'static>(props: FlowProps<T>) -> Element {
    let mut state = props.state;
    let mut dragging_node: Signal<Option<(NodeId, Position)>> = use_signal(|| None);
    let mut is_panning: Signal<bool> = use_signal(|| false);
    let mut last_mouse_pos: Signal<Option<Position>> = use_signal(|| None);

    let viewport = state.read().viewport;
    let transform = format!(
        "translate({}px, {}px) scale({})",
        viewport.x, viewport.y, viewport.zoom
    );

    // Handle mouse move for dragging and panning
    let on_mouse_move = {
        let on_node_drag = props.on_node_drag.clone();
        move |evt: MouseEvent| {
            let coords = evt.client_coordinates();
            let current_pos = Position::new(coords.x, coords.y);

            // Handle node dragging
            let dragging_info = dragging_node.read().clone();
            if let Some((node_id, start_pos)) = dragging_info {
                let zoom = state.read().viewport.zoom;
                let dx = (current_pos.x - start_pos.x) / zoom;
                let dy = (current_pos.y - start_pos.y) / zoom;

                let node_pos = state.read().get_node(&node_id).map(|n| n.position);
                if let Some(pos) = node_pos {
                    let new_pos = Position::new(pos.x + dx, pos.y + dy);
                    state.write().update_node_position(&node_id, new_pos);

                    if let Some(handler) = &on_node_drag {
                        handler.call((node_id.clone(), new_pos));
                    }
                }

                dragging_node.set(Some((node_id, current_pos)));
            }

            // Handle panning
            if *is_panning.read() {
                if let Some(last_pos) = *last_mouse_pos.read() {
                    let dx = current_pos.x - last_pos.x;
                    let dy = current_pos.y - last_pos.y;
                    state.write().pan(dx, dy);
                }
                last_mouse_pos.set(Some(current_pos));
            }

            // Update connection line if connecting
            if state.read().connection.is_some() {
                let vp = state.read().viewport;
                let flow_pos = vp.screen_to_flow(current_pos.x, current_pos.y);
                state.write().update_connection(flow_pos);
            }
        }
    };

    // Handle mouse up - end dragging/panning
    let on_mouse_up = move |_evt: MouseEvent| {
        if dragging_node.read().is_some() {
            dragging_node.set(None);
        }
        if *is_panning.read() {
            is_panning.set(false);
            last_mouse_pos.set(None);
        }
        // Cancel connection if not completed
        if state.read().connection.is_some() {
            state.write().cancel_connection();
        }
    };

    // Handle mouse down on pane - start panning
    let pan_on_drag = props.pan_on_drag;
    let on_pane_click = props.on_pane_click.clone();
    let on_mouse_down = move |evt: MouseEvent| {
        if pan_on_drag {
            let coords = evt.client_coordinates();
            is_panning.set(true);
            last_mouse_pos.set(Some(Position::new(coords.x, coords.y)));
        }

        // Clear selection
        state.write().clear_selection();

        if let Some(handler) = &on_pane_click {
            let coords = evt.client_coordinates();
            let vp = state.read().viewport;
            let flow_pos = vp.screen_to_flow(coords.x, coords.y);
            handler.call(flow_pos);
        }
    };

    // Handle wheel for zooming
    let min_zoom = props.min_zoom;
    let max_zoom = props.max_zoom;
    let zoom_on_scroll = props.zoom_on_scroll;
    let on_viewport_change = props.on_viewport_change.clone();
    let on_wheel = move |evt: WheelEvent| {
        if !zoom_on_scroll {
            return;
        }

        evt.prevent_default();
        let wheel_delta = evt.delta();
        let delta_y = match wheel_delta {
            WheelDelta::Pixels(p) => p.y,
            WheelDelta::Lines(l) => l.y * 20.0,
            WheelDelta::Pages(p) => p.y * 100.0,
        };
        let delta = -delta_y / 500.0;
        let coords = evt.client_coordinates();

        let mut state_mut = state.write();
        let old_zoom = state_mut.viewport.zoom;
        let new_zoom = (old_zoom + delta).clamp(min_zoom, max_zoom);

        // Zoom around mouse position
        state_mut.viewport.x =
            coords.x - (coords.x - state_mut.viewport.x) * new_zoom / old_zoom;
        state_mut.viewport.y =
            coords.y - (coords.y - state_mut.viewport.y) * new_zoom / old_zoom;
        state_mut.viewport.zoom = new_zoom;

        if let Some(handler) = &on_viewport_change {
            handler.call(state_mut.viewport);
        }
    };

    // Node event handlers
    let on_node_click = props.on_node_click.clone();
    let nodes_selectable = props.nodes_selectable;
    let on_node_select = move |node_id: NodeId| {
        if nodes_selectable {
            state.write().select_node(&node_id, false);
        }
        if let Some(handler) = &on_node_click {
            handler.call(node_id);
        }
    };

    let nodes_draggable = props.nodes_draggable;
    let on_node_drag_start = move |(node_id, pos): (NodeId, Position)| {
        if nodes_draggable {
            if let Some(node) = state.read().get_node(&node_id) {
                if node.draggable {
                    dragging_node.set(Some((node_id, pos)));
                }
            }
        }
    };

    let nodes_connectable = props.nodes_connectable;
    let on_connect = props.on_connect.clone();
    let on_connect_start = move |(node_id, handle_pos): (NodeId, HandlePosition)| {
        if !nodes_connectable {
            return;
        }
        let node_info = state.read().get_node(&node_id).map(|n| (n.connectable, n.handle_position(handle_pos)));
        if let Some((connectable, source_pos)) = node_info {
            if connectable {
                state.write().start_connection(node_id, handle_pos, source_pos);
            }
        }
    };

    let on_connect_end = move |(node_id, handle_pos): (NodeId, HandlePosition)| {
        if !nodes_connectable {
            return;
        }
        if let Some(edge) = state.write().complete_connection(node_id, handle_pos) {
            if let Some(handler) = &on_connect {
                handler.call(edge);
            }
        }
    };

    // Edge event handlers
    let on_edge_click = props.on_edge_click.clone();
    let edges_selectable = props.edges_selectable;
    let on_edge_select = move |edge_id: String| {
        if edges_selectable {
            state.write().select_edge(&edge_id, false);
        }
        if let Some(handler) = &on_edge_click {
            handler.call(edge_id);
        }
    };

    // Read state for rendering
    let nodes = state.read().nodes.clone();
    let edges = state.read().edges.clone();
    let connection = state.read().connection.clone();
    let current_zoom = state.read().viewport.zoom;

    rsx! {
        div {
            class: "dioxus-flow-container {props.class}",
            style: "width: 100%; height: 100%; position: relative; overflow: hidden;",
            onmousedown: on_mouse_down,
            onmousemove: on_mouse_move,
            onmouseup: on_mouse_up,
            onmouseleave: on_mouse_up,
            onwheel: on_wheel,

            // SVG layer for edges
            svg {
                class: "dioxus-flow-edges",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none;",

                // Defs for markers
                defs {
                    marker {
                        id: "dioxus-flow-arrowhead",
                        marker_width: "12",
                        marker_height: "12",
                        ref_x: "10",
                        ref_y: "6",
                        orient: "auto",
                        polygon {
                            points: "0 0, 12 6, 0 12",
                            fill: "#b1b1b7",
                        }
                    }
                }

                g {
                    style: "transform: {transform};",
                    // Render edges
                    for edge in edges.iter() {
                        {
                            let source_node = state.read().get_node(&edge.source).cloned();
                            let target_node = state.read().get_node(&edge.target).cloned();

                            if let (Some(source), Some(target)) = (source_node, target_node) {
                                let source_pos = source.handle_position(edge.source_handle);
                                let target_pos = target.handle_position(edge.target_handle);

                                rsx! {
                                    EdgeComponent {
                                        key: "{edge.id}",
                                        edge: edge.clone(),
                                        source_position: source_pos,
                                        target_position: target_pos,
                                        on_select: on_edge_select,
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                    }

                    // Render connection line when connecting
                    if let Some(conn) = connection {
                        {
                            if let Some(source_node) = state.read().get_node(&conn.source).cloned() {
                                let source_pos = source_node.handle_position(conn.source_handle);
                                rsx! {
                                    ConnectionLine {
                                        source: source_pos,
                                        source_handle: conn.source_handle,
                                        target: conn.target_position,
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                    }
                }
            }

            // Nodes layer
            div {
                class: "dioxus-flow-nodes",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; transform: {transform}; transform-origin: 0 0;",

                for node in nodes.iter() {
                    NodeComponent {
                        key: "{node.id}",
                        node: node.clone(),
                        zoom: current_zoom,
                        dragging: dragging_node.read().as_ref().map(|(id, _)| id == &node.id).unwrap_or(false),
                        on_select: on_node_select,
                        on_drag_start: on_node_drag_start,
                        on_connect_start: on_connect_start,
                        on_connect_end: on_connect_end,
                    }
                }
            }

            // Additional children
            {props.children}
        }
    }
}

/// Default CSS styles for the flow.
pub const FLOW_STYLES: &str = r#"
.dioxus-flow-container {
    background-color: #f8f8f8;
    background-image: radial-gradient(#ddd 1px, transparent 1px);
    background-size: 20px 20px;
}

.dioxus-flow-node {
    position: absolute;
    padding: 10px 20px;
    border-radius: 5px;
    background: white;
    border: 1px solid #ddd;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
    cursor: grab;
    user-select: none;
    min-width: 150px;
    text-align: center;
}

.dioxus-flow-node:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.dioxus-flow-node-selected {
    border-color: #1a192b;
    box-shadow: 0 0 0 0.5px #1a192b;
}

.dioxus-flow-node-dragging {
    cursor: grabbing;
    opacity: 0.8;
}

.dioxus-flow-handle {
    position: absolute;
    width: 10px;
    height: 10px;
    background: #1a192b;
    border-radius: 50%;
    border: 2px solid white;
}

.dioxus-flow-handle-top {
    top: -5px;
    left: 50%;
    transform: translateX(-50%);
}

.dioxus-flow-handle-right {
    top: 50%;
    right: -5px;
    transform: translateY(-50%);
}

.dioxus-flow-handle-bottom {
    bottom: -5px;
    left: 50%;
    transform: translateX(-50%);
}

.dioxus-flow-handle-left {
    top: 50%;
    left: -5px;
    transform: translateY(-50%);
}

.dioxus-flow-handle-source {
    cursor: crosshair;
}

.dioxus-flow-handle-target {
    cursor: crosshair;
}

.dioxus-flow-edge {
    pointer-events: all;
}

.dioxus-flow-edge-path {
    transition: stroke 0.2s;
}

.dioxus-flow-edge-selected .dioxus-flow-edge-path {
    stroke: #1a192b;
}

.dioxus-flow-edge-animated .dioxus-flow-edge-path {
    stroke-dasharray: 5;
    animation: dioxus-flow-dash 0.5s linear infinite;
}

@keyframes dioxus-flow-dash {
    to {
        stroke-dashoffset: -10;
    }
}

.dioxus-flow-edge-label {
    background: white;
    padding: 2px 4px;
    border-radius: 3px;
    font-size: 12px;
    text-align: center;
}

.dioxus-flow-connection-line {
    pointer-events: none;
}
"#;

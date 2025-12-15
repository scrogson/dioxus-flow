//! Main Flow component.

use crate::components::edge::{ConnectionLine, EdgeComponent};
use crate::components::node::NodeComponent;
use crate::hooks::FlowState;
use crate::types::{Edge, FlowEvent, HandlePosition, NodeId, Position, SelectionRect, Viewport};
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

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
    /// Whether to pan on scroll (instead of zoom).
    #[props(default = false)]
    pub pan_on_scroll: bool,
    /// Whether zooming is enabled.
    #[props(default = true)]
    pub zoom_on_scroll: bool,
    /// Whether zoom on double-click is enabled.
    #[props(default = true)]
    pub zoom_on_double_click: bool,
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
    /// Whether elements can be deleted with keyboard.
    #[props(default = true)]
    pub elements_deletable: bool,
    /// Whether multi-select is enabled (shift+click, box select).
    #[props(default = true)]
    pub multi_select: bool,
    /// Whether box selection on drag is enabled.
    #[props(default = false)]
    pub selection_on_drag: bool,
    /// Callback for node click.
    #[props(default)]
    pub on_node_click: Option<EventHandler<NodeId>>,
    /// Callback for node double-click.
    #[props(default)]
    pub on_node_double_click: Option<EventHandler<NodeId>>,
    /// Callback for node mouse enter.
    #[props(default)]
    pub on_node_mouse_enter: Option<EventHandler<NodeId>>,
    /// Callback for node mouse leave.
    #[props(default)]
    pub on_node_mouse_leave: Option<EventHandler<NodeId>>,
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
    /// Callback when nodes are deleted.
    #[props(default)]
    pub on_nodes_delete: Option<EventHandler<Vec<NodeId>>>,
    /// Callback when edges are deleted.
    #[props(default)]
    pub on_edges_delete: Option<EventHandler<Vec<String>>>,
    /// Callback for selection change.
    #[props(default)]
    pub on_selection_change: Option<EventHandler<(Vec<NodeId>, Vec<String>)>>,
    /// Additional CSS class for the container.
    #[props(default)]
    pub class: String,
    /// Custom node content renderer. Receives the node and should return the inner content.
    #[props(default)]
    pub node_render: Option<Callback<crate::types::Node<T>, Element>>,
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
    let mut selection_box: Signal<Option<(Position, Position)>> = use_signal(|| None);
    let mut shift_held: Signal<bool> = use_signal(|| false);
    let mut last_click_time: Signal<f64> = use_signal(|| 0.0);
    let mut last_click_node: Signal<Option<NodeId>> = use_signal(|| None);

    // Touch state
    let mut touch_start: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut pinch_distance: Signal<Option<f64>> = use_signal(|| None);

    let viewport = state.read().viewport;
    let transform = format!(
        "translate({}px, {}px) scale({})",
        viewport.x, viewport.y, viewport.zoom
    );

    // Handle keyboard events
    let elements_deletable = props.elements_deletable;
    let on_nodes_delete = props.on_nodes_delete.clone();
    let on_edges_delete = props.on_edges_delete.clone();
    let on_selection_change = props.on_selection_change.clone();

    let on_key_down = move |evt: KeyboardEvent| {
        let key = evt.key();
        let ctrl_or_meta = evt.modifiers().meta() || evt.modifiers().ctrl();
        let key_str = format!("{:?}", key);

        match key_str.as_str() {
            "Backspace" | "Delete" => {
                if elements_deletable {
                    state.write().save_to_history();
                    let (deleted_nodes, deleted_edges) = state.write().delete_selected();
                    if !deleted_nodes.is_empty() {
                        if let Some(handler) = &on_nodes_delete {
                            handler.call(deleted_nodes);
                        }
                    }
                    if !deleted_edges.is_empty() {
                        if let Some(handler) = &on_edges_delete {
                            handler.call(deleted_edges);
                        }
                    }
                }
            }
            "Escape" => {
                state.write().cancel_connection();
                state.write().clear_selection();
                selection_box.set(None);
            }
            "a" if ctrl_or_meta => {
                evt.prevent_default();
                state.write().select_all();
                let selected_nodes = state.read().selected_nodes.clone();
                let selected_edges = state.read().selected_edges.clone();
                if let Some(handler) = &on_selection_change {
                    handler.call((selected_nodes, selected_edges));
                }
            }
            "c" if ctrl_or_meta => {
                evt.prevent_default();
                state.write().copy_selected();
            }
            "x" if ctrl_or_meta => {
                evt.prevent_default();
                state.write().save_to_history();
                let (deleted_nodes, deleted_edges) = state.write().cut_selected();
                if !deleted_nodes.is_empty() {
                    if let Some(handler) = &on_nodes_delete {
                        handler.call(deleted_nodes);
                    }
                }
                if !deleted_edges.is_empty() {
                    if let Some(handler) = &on_edges_delete {
                        handler.call(deleted_edges);
                    }
                }
            }
            "v" if ctrl_or_meta => {
                evt.prevent_default();
                state.write().save_to_history();
                state.write().paste(Position::new(20.0, 20.0));
            }
            "z" if ctrl_or_meta && evt.modifiers().shift() => {
                evt.prevent_default();
                state.write().redo();
            }
            "z" if ctrl_or_meta => {
                evt.prevent_default();
                state.write().undo();
            }
            "y" if ctrl_or_meta => {
                evt.prevent_default();
                state.write().redo();
            }
            "ArrowUp" => {
                evt.prevent_default();
                let delta = if evt.modifiers().shift() { 10.0 } else { 1.0 };
                state.write().move_selected_nodes(0.0, -delta);
            }
            "ArrowDown" => {
                evt.prevent_default();
                let delta = if evt.modifiers().shift() { 10.0 } else { 1.0 };
                state.write().move_selected_nodes(0.0, delta);
            }
            "ArrowLeft" => {
                evt.prevent_default();
                let delta = if evt.modifiers().shift() { 10.0 } else { 1.0 };
                state.write().move_selected_nodes(-delta, 0.0);
            }
            "ArrowRight" => {
                evt.prevent_default();
                let delta = if evt.modifiers().shift() { 10.0 } else { 1.0 };
                state.write().move_selected_nodes(delta, 0.0);
            }
            "Shift" => {
                shift_held.set(true);
            }
            _ => {}
        }
    };

    let on_key_up = move |evt: KeyboardEvent| {
        let key_str = format!("{:?}", evt.key());
        if key_str == "Shift" {
            shift_held.set(false);
        }
    };

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
                    // Emit event for centralized handling
                    state.write().emit_event(FlowEvent::NodeDrag {
                        id: node_id.clone(),
                        position: new_pos,
                    });
                }

                dragging_node.set(Some((node_id, current_pos)));
                return;
            }

            // Handle box selection
            let selection_box_val = *selection_box.read();
            if let Some((start, _)) = selection_box_val {
                let vp = state.read().viewport;
                let end_flow = vp.screen_to_flow(current_pos.x, current_pos.y);
                selection_box.set(Some((start, end_flow)));
                return;
            }

            // Handle panning
            if *is_panning.read() {
                if let Some(last_pos) = *last_mouse_pos.read() {
                    let dx = current_pos.x - last_pos.x;
                    let dy = current_pos.y - last_pos.y;
                    state.write().pan(dx, dy);
                }
                last_mouse_pos.set(Some(current_pos));
                return;
            }

            // Update connection line if connecting
            if state.read().connection.is_some() {
                let vp = state.read().viewport;
                let flow_pos = vp.screen_to_flow(current_pos.x, current_pos.y);
                state.write().update_connection(flow_pos);
            }
        }
    };

    // Handle mouse up - end dragging/panning/selection
    let multi_select = props.multi_select;
    let on_mouse_up = {
        let on_selection_change = props.on_selection_change.clone();
        move |_evt: MouseEvent| {
            // Complete box selection
            let selection_box_val = *selection_box.read();
            if let Some((start, end)) = selection_box_val {
                let min_x = start.x.min(end.x);
                let min_y = start.y.min(end.y);
                let max_x = start.x.max(end.x);
                let max_y = start.y.max(end.y);

                let rect = SelectionRect {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                };

                let shift = *shift_held.read();
                state
                    .write()
                    .select_in_rect(rect, shift && multi_select);

                let selected_nodes = state.read().selected_nodes.clone();
                let selected_edges = state.read().selected_edges.clone();
                if let Some(handler) = &on_selection_change {
                    handler.call((selected_nodes, selected_edges));
                }

                selection_box.set(None);
            }

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
        }
    };

    // Handle mouse down on pane - start panning or box selection
    let pan_on_drag = props.pan_on_drag;
    let selection_on_drag = props.selection_on_drag;
    let on_pane_click = props.on_pane_click.clone();
    let zoom_on_double_click = props.zoom_on_double_click;
    let min_zoom = props.min_zoom;
    let max_zoom = props.max_zoom;
    let on_mouse_down = move |evt: MouseEvent| {
        let coords = evt.client_coordinates();
        let current_pos = Position::new(coords.x, coords.y);

        // Check for double-click (zoom)
        let now = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        if zoom_on_double_click && now - *last_click_time.read() < 300.0 {
            // Double-click: zoom in
            let vp = state.read().viewport;
            let new_zoom = (vp.zoom * 1.5).min(max_zoom);
            state.write().set_zoom(new_zoom, coords.x, coords.y);
            last_click_time.set(0.0);
            return;
        }
        last_click_time.set(now);
        last_click_node.set(None);

        // Start box selection if shift is held or selection_on_drag is enabled
        if selection_on_drag || (*shift_held.read() && !pan_on_drag) {
            let vp = state.read().viewport;
            let flow_pos = vp.screen_to_flow(coords.x, coords.y);
            selection_box.set(Some((flow_pos, flow_pos)));
            return;
        }

        // Start panning
        if pan_on_drag {
            is_panning.set(true);
            last_mouse_pos.set(Some(current_pos));
        }

        // Clear selection (unless shift is held)
        if !*shift_held.read() {
            state.write().clear_selection();
        }

        if let Some(handler) = &on_pane_click {
            let vp = state.read().viewport;
            let flow_pos = vp.screen_to_flow(coords.x, coords.y);
            handler.call(flow_pos);
        }
    };

    // Handle wheel for zooming or panning
    let zoom_on_scroll = props.zoom_on_scroll;
    let pan_on_scroll = props.pan_on_scroll;
    let on_viewport_change = props.on_viewport_change.clone();
    let on_wheel = move |evt: WheelEvent| {
        evt.prevent_default();
        let wheel_delta = evt.delta();
        let delta_y = match wheel_delta {
            WheelDelta::Pixels(p) => p.y,
            WheelDelta::Lines(l) => l.y * 20.0,
            WheelDelta::Pages(p) => p.y * 100.0,
        };
        let delta_x = match wheel_delta {
            WheelDelta::Pixels(p) => p.x,
            WheelDelta::Lines(l) => l.x * 20.0,
            WheelDelta::Pages(p) => p.x * 100.0,
        };

        if pan_on_scroll {
            // Pan instead of zoom
            state.write().pan(-delta_x, -delta_y);
        } else if zoom_on_scroll {
            // Zoom
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
        }
    };

    // Touch event handlers
    let on_touch_start = move |evt: TouchEvent| {
        let touches = evt.touches();
        if touches.len() == 1 {
            // Single touch - start panning
            if let Some(touch) = touches.first() {
                touch_start.set(Some((
                    touch.client_coordinates().x,
                    touch.client_coordinates().y,
                )));
                is_panning.set(true);
            }
        } else if touches.len() == 2 {
            // Two touches - start pinch zoom
            if let (Some(t1), Some(t2)) = (touches.first(), touches.get(1)) {
                let dx = t1.client_coordinates().x - t2.client_coordinates().x;
                let dy = t1.client_coordinates().y - t2.client_coordinates().y;
                let distance = (dx * dx + dy * dy).sqrt();
                pinch_distance.set(Some(distance));
            }
        }
    };

    let on_touch_move = move |evt: TouchEvent| {
        let touches = evt.touches();
        if touches.len() == 1 && *is_panning.read() {
            // Single touch panning
            if let Some(touch) = touches.first() {
                let current = (
                    touch.client_coordinates().x,
                    touch.client_coordinates().y,
                );
                let start_val = *touch_start.read();
                if let Some(start) = start_val {
                    let dx = current.0 - start.0;
                    let dy = current.1 - start.1;
                    state.write().pan(dx, dy);
                    touch_start.set(Some(current));
                }
            }
        } else if touches.len() == 2 {
            // Pinch zoom
            if let (Some(t1), Some(t2)) = (touches.first(), touches.get(1)) {
                let dx = t1.client_coordinates().x - t2.client_coordinates().x;
                let dy = t1.client_coordinates().y - t2.client_coordinates().y;
                let distance = (dx * dx + dy * dy).sqrt();

                if let Some(prev_distance) = *pinch_distance.read() {
                    let scale = distance / prev_distance;
                    let center_x =
                        (t1.client_coordinates().x + t2.client_coordinates().x) / 2.0;
                    let center_y =
                        (t1.client_coordinates().y + t2.client_coordinates().y) / 2.0;

                    let mut state_mut = state.write();
                    let old_zoom = state_mut.viewport.zoom;
                    let new_zoom = (old_zoom * scale).clamp(min_zoom, max_zoom);

                    state_mut.viewport.x =
                        center_x - (center_x - state_mut.viewport.x) * new_zoom / old_zoom;
                    state_mut.viewport.y =
                        center_y - (center_y - state_mut.viewport.y) * new_zoom / old_zoom;
                    state_mut.viewport.zoom = new_zoom;
                }
                pinch_distance.set(Some(distance));
            }
        }
    };

    let on_touch_end = move |_evt: TouchEvent| {
        touch_start.set(None);
        pinch_distance.set(None);
        is_panning.set(false);
    };

    // Node event handlers
    let on_node_click = props.on_node_click.clone();
    let on_node_double_click = props.on_node_double_click.clone();
    let nodes_selectable = props.nodes_selectable;
    let on_node_select = {
        let on_selection_change = props.on_selection_change.clone();
        move |node_id: NodeId| {
            // Check for double-click
            let now = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);
            let last_click_t = *last_click_time.read();
            let last_node_val = last_click_node.read().clone();
            if now - last_click_t < 300.0 {
                if let Some(last_node) = last_node_val {
                    if last_node == node_id {
                        if let Some(handler) = &on_node_double_click {
                            handler.call(node_id.clone());
                        }
                        last_click_time.set(0.0);
                        last_click_node.set(None);
                        return;
                    }
                }
            }
            last_click_time.set(now);
            last_click_node.set(Some(node_id.clone()));

            if nodes_selectable {
                let multi = *shift_held.read() && multi_select;
                state.write().select_node(&node_id, multi);
                // Bring to front
                state.write().bring_to_front(&node_id);

                let selected_nodes = state.read().selected_nodes.clone();
                let selected_edges = state.read().selected_edges.clone();
                if let Some(handler) = &on_selection_change {
                    handler.call((selected_nodes, selected_edges));
                }
            }
            if let Some(handler) = &on_node_click {
                handler.call(node_id.clone());
            }
            // Emit event for centralized handling
            state.write().emit_event(FlowEvent::NodeClick(node_id));
        }
    };

    let nodes_draggable = props.nodes_draggable;
    let on_node_drag_start = move |(node_id, pos): (NodeId, Position)| {
        if nodes_draggable {
            let is_draggable = state.read().get_node(&node_id).map(|n| n.draggable).unwrap_or(false);
            if is_draggable {
                state.write().save_to_history();
                dragging_node.set(Some((node_id, pos)));
            }
        }
    };

    let nodes_connectable = props.nodes_connectable;
    let on_connect = props.on_connect.clone();
    let on_connect_start = move |(node_id, handle_pos): (NodeId, HandlePosition)| {
        if !nodes_connectable {
            return;
        }
        let node_info = state
            .read()
            .get_node(&node_id)
            .map(|n| (n.connectable, n.handle_position(handle_pos)));
        if let Some((connectable, source_pos)) = node_info {
            if connectable {
                state
                    .write()
                    .start_connection(node_id, handle_pos, source_pos);
            }
        }
    };

    let on_connect_end = move |(node_id, handle_pos): (NodeId, HandlePosition)| {
        if !nodes_connectable {
            return;
        }
        if state.read().connection.is_some() {
            state.write().save_to_history();
        }
        let edge = state.write().complete_connection(node_id.clone(), handle_pos);
        if let Some(edge) = edge {
            if let Some(handler) = &on_connect {
                handler.call(edge.clone());
            }
            // Emit event for centralized handling
            state.write().emit_event(FlowEvent::Connect {
                source: edge.source.clone(),
                source_handle: edge.source_handle,
                target: edge.target.clone(),
                target_handle: edge.target_handle,
            });
        }
    };

    // Edge event handlers
    let on_edge_click = props.on_edge_click.clone();
    let edges_selectable = props.edges_selectable;
    let on_edge_select = {
        let on_selection_change = props.on_selection_change.clone();
        move |edge_id: String| {
            if edges_selectable {
                let multi = *shift_held.read() && multi_select;
                state.write().select_edge(&edge_id, multi);

                let selected_nodes = state.read().selected_nodes.clone();
                let selected_edges = state.read().selected_edges.clone();
                if let Some(handler) = &on_selection_change {
                    handler.call((selected_nodes, selected_edges));
                }
            }
            if let Some(handler) = &on_edge_click {
                handler.call(edge_id.clone());
            }
            // Emit event for centralized handling
            state.write().emit_event(FlowEvent::EdgeClick(edge_id));
        }
    };

    // Read state for rendering - sort nodes by z-index
    let nodes = state
        .read()
        .nodes_sorted_by_z_index()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let edges = state.read().edges.clone();
    let connection = state.read().connection.clone();
    let current_zoom = state.read().viewport.zoom;

    // Calculate selection box rect for rendering
    let selection_rect: Option<(f64, f64, f64, f64)> = (*selection_box.read()).map(|(start, end)| {
        let vp = state.read().viewport;
        let start_screen = vp.flow_to_screen(start.x, start.y);
        let end_screen = vp.flow_to_screen(end.x, end.y);
        (
            start_screen.x.min(end_screen.x),
            start_screen.y.min(end_screen.y),
            (end_screen.x - start_screen.x).abs(),
            (end_screen.y - start_screen.y).abs(),
        )
    });

    rsx! {
        div {
            class: "dioxus-flow-container {props.class}",
            style: "width: 100%; height: 100%; position: absolute; top: 0; left: 0; overflow: hidden; outline: none; z-index: 1;",
            tabindex: "0",
            onkeydown: on_key_down,
            onkeyup: on_key_up,
            onmousedown: on_mouse_down,
            onmousemove: on_mouse_move,
            onmouseup: on_mouse_up,
            onmouseleave: on_mouse_up,
            onwheel: on_wheel,
            ontouchstart: on_touch_start,
            ontouchmove: on_touch_move,
            ontouchend: on_touch_end,

            // SVG layer for edges
            svg {
                class: "dioxus-flow-edges",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none;",

                // Defs for markers - arrow tip at endpoint
                defs {
                    marker {
                        id: "dioxus-flow-arrowhead",
                        view_box: "0 0 10 10",
                        marker_width: "10",
                        marker_height: "10",
                        ref_x: "10",
                        ref_y: "5",
                        orient: "auto-start-reverse",
                        marker_units: "userSpaceOnUse",
                        path {
                            d: "M 0 0 L 10 5 L 0 10 z",
                            fill: "#64748b",
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
                                // Try to get position and direction from handle ID first, fall back to handle position
                                let (source_pos, source_dir) = edge.source_handle_id.as_ref()
                                    .and_then(|id| source.handle_info_by_id(id))
                                    .unwrap_or_else(|| (source.handle_position(edge.source_handle), edge.source_handle));
                                let (target_pos, target_dir) = edge.target_handle_id.as_ref()
                                    .and_then(|id| target.handle_info_by_id(id))
                                    .unwrap_or_else(|| (target.handle_position(edge.target_handle), edge.target_handle));

                                rsx! {
                                    EdgeComponent {
                                        key: "{edge.id}",
                                        edge: edge.clone(),
                                        source_position: source_pos,
                                        target_position: target_pos,
                                        source_handle_direction: source_dir,
                                        target_handle_direction: target_dir,
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
                                let source_pos = conn.source_handle_id.as_ref()
                                    .and_then(|id| source_node.handle_position_by_id(id))
                                    .unwrap_or_else(|| source_node.handle_position(conn.source_handle));
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

            // Nodes layer - pointer-events: none so clicks pass through to container for panning
            div {
                class: "dioxus-flow-nodes",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; transform: {transform}; transform-origin: 0 0; pointer-events: none;",

                for node in nodes.iter() {
                    {
                        let custom_content = props.node_render.as_ref().map(|render| render.call(node.clone()));
                        rsx! {
                            NodeComponent {
                                key: "{node.id}",
                                node: node.clone(),
                                zoom: current_zoom,
                                dragging: dragging_node.read().as_ref().map(|(id, _)| id == &node.id).unwrap_or(false),
                                on_select: on_node_select,
                                on_drag_start: on_node_drag_start,
                                on_connect_start: on_connect_start,
                                on_connect_end: on_connect_end,
                                {custom_content}
                            }
                        }
                    }
                }
            }

            // Selection box overlay
            if let Some((x, y, w, h)) = selection_rect {
                div {
                    class: "dioxus-flow-selection-box",
                    style: "position: absolute; left: {x}px; top: {y}px; width: {w}px; height: {h}px; border: 1px dashed #1a192b; background: rgba(26, 25, 43, 0.08); pointer-events: none;",
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

.dioxus-flow-container:focus {
    outline: none;
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
    min-height: 40px;
    text-align: center;
    box-sizing: border-box;
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

.dioxus-flow-selection-box {
    z-index: 9999;
}
"#;

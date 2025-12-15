//! State management hooks for dioxus-flow.

use crate::types::{
    ClipboardData, Connection, ConnectionValidation, DefaultEdgeOptions, Edge, EdgeId, FlowEvent,
    Node, NodeId, PendingConnection, Position, SelectionRect, SnapGrid, Viewport,
};
use dioxus::prelude::*;
use std::collections::HashMap;

/// Maximum history size for undo/redo.
const MAX_HISTORY_SIZE: usize = 100;

/// A snapshot of the flow state for undo/redo.
#[derive(Debug, Clone)]
pub struct FlowSnapshot<T: Clone + PartialEq + 'static> {
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
}

/// Flow state containing all nodes, edges, and viewport information.
#[derive(Debug, Clone)]
pub struct FlowState<T: Clone + PartialEq + 'static = ()> {
    /// All nodes in the flow.
    pub nodes: Vec<Node<T>>,
    /// All edges in the flow.
    pub edges: Vec<Edge>,
    /// Current viewport.
    pub viewport: Viewport,
    /// Currently selected node IDs.
    pub selected_nodes: Vec<NodeId>,
    /// Currently selected edge IDs.
    pub selected_edges: Vec<EdgeId>,
    /// Active connection being created.
    pub connection: Option<Connection>,
    /// Node dimensions cache.
    pub node_dimensions: HashMap<NodeId, (f64, f64)>,
    /// Snap grid configuration.
    pub snap_grid: SnapGrid,
    /// Default edge options.
    pub default_edge_options: DefaultEdgeOptions,
    /// Clipboard data.
    pub clipboard: ClipboardData<T>,
    /// Undo history.
    pub undo_stack: Vec<FlowSnapshot<T>>,
    /// Redo history.
    pub redo_stack: Vec<FlowSnapshot<T>>,
    /// Maximum z-index used (for bringing nodes to front).
    pub max_z_index: i32,
    /// Connection validator function result cache.
    pub connection_valid: bool,
}

impl<T: Clone + Default + PartialEq + 'static> Default for FlowState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Default + PartialEq + 'static> FlowState<T> {
    /// Create a new empty flow state.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            viewport: Viewport::default(),
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            connection: None,
            node_dimensions: HashMap::new(),
            snap_grid: SnapGrid::default(),
            default_edge_options: DefaultEdgeOptions::default(),
            clipboard: ClipboardData::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_z_index: 0,
            connection_valid: true,
        }
    }

    /// Create flow state with initial nodes and edges.
    pub fn with_nodes_and_edges(nodes: Vec<Node<T>>, edges: Vec<Edge>) -> Self {
        let max_z = nodes.iter().map(|n| n.z_index).max().unwrap_or(0);
        Self {
            nodes,
            edges,
            viewport: Viewport::default(),
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            connection: None,
            node_dimensions: HashMap::new(),
            snap_grid: SnapGrid::default(),
            default_edge_options: DefaultEdgeOptions::default(),
            clipboard: ClipboardData::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_z_index: max_z,
            connection_valid: true,
        }
    }

    /// Save current state to undo history.
    pub fn save_to_history(&mut self) {
        let snapshot = FlowSnapshot {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        };
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
        // Clear redo stack when new action is performed
        self.redo_stack.clear();
    }

    /// Undo the last action.
    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current = FlowSnapshot {
                nodes: self.nodes.clone(),
                edges: self.edges.clone(),
            };
            self.redo_stack.push(current);

            // Restore previous state
            self.nodes = snapshot.nodes;
            self.edges = snapshot.edges;
            self.clear_selection();
            true
        } else {
            false
        }
    }

    /// Redo the last undone action.
    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current = FlowSnapshot {
                nodes: self.nodes.clone(),
                edges: self.edges.clone(),
            };
            self.undo_stack.push(current);

            // Restore next state
            self.nodes = snapshot.nodes;
            self.edges = snapshot.edges;
            self.clear_selection();
            true
        } else {
            false
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &str) -> Option<&Node<T>> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node<T>> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Get an edge by ID.
    pub fn get_edge(&self, id: &str) -> Option<&Edge> {
        self.edges.iter().find(|e| e.id == id)
    }

    /// Get a mutable edge by ID.
    pub fn get_edge_mut(&mut self, id: &str) -> Option<&mut Edge> {
        self.edges.iter_mut().find(|e| e.id == id)
    }

    /// Add a node to the flow.
    pub fn add_node(&mut self, mut node: Node<T>) {
        // Assign z-index if not set
        if node.z_index == 0 {
            self.max_z_index += 1;
            node.z_index = self.max_z_index;
        } else {
            self.max_z_index = self.max_z_index.max(node.z_index);
        }
        self.nodes.push(node);
    }

    /// Remove a node and all connected edges.
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.source != id && e.target != id);
        self.selected_nodes.retain(|n| n != id);
    }

    /// Add an edge to the flow.
    pub fn add_edge(&mut self, edge: Edge) {
        // Check if edge already exists
        let exists = self.edges.iter().any(|e| {
            e.source == edge.source
                && e.target == edge.target
                && e.source_handle == edge.source_handle
                && e.target_handle == edge.target_handle
        });
        if !exists {
            self.edges.push(edge);
        }
    }

    /// Remove an edge by ID.
    pub fn remove_edge(&mut self, id: &str) {
        self.edges.retain(|e| e.id != id);
        self.selected_edges.retain(|e| e != id);
    }

    /// Delete all selected nodes and edges.
    pub fn delete_selected(&mut self) -> (Vec<NodeId>, Vec<EdgeId>) {
        let deleted_nodes: Vec<NodeId> = self
            .selected_nodes
            .iter()
            .filter(|id| {
                self.get_node(id)
                    .map(|n| n.deletable)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        let deleted_edges: Vec<EdgeId> = self
            .selected_edges
            .iter()
            .filter(|id| {
                self.get_edge(id)
                    .map(|e| e.deletable)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Also delete edges connected to deleted nodes
        let edges_to_delete: Vec<EdgeId> = self
            .edges
            .iter()
            .filter(|e| deleted_nodes.contains(&e.source) || deleted_nodes.contains(&e.target))
            .map(|e| e.id.clone())
            .collect();

        for id in &deleted_nodes {
            self.remove_node(id);
        }

        for id in &deleted_edges {
            self.remove_edge(id);
        }

        for id in &edges_to_delete {
            self.remove_edge(id);
        }

        self.clear_selection();

        let mut all_deleted_edges = deleted_edges;
        all_deleted_edges.extend(edges_to_delete);
        all_deleted_edges.sort();
        all_deleted_edges.dedup();

        (deleted_nodes, all_deleted_edges)
    }

    /// Update a node's position with optional snap-to-grid.
    pub fn update_node_position(&mut self, id: &str, position: Position) {
        // Read snap_grid values before borrowing node mutably
        let snap_enabled = self.snap_grid.enabled;
        let snapped_pos = self.snap_grid.snap(position);

        if let Some(node) = self.get_node_mut(id) {
            let mut new_pos = if snap_enabled {
                snapped_pos
            } else {
                position
            };

            // Apply extent constraints if set
            if let Some(extent) = node.extent {
                let w = node.width.unwrap_or(150.0);
                let h = node.height.unwrap_or(40.0);
                new_pos = extent.clamp(new_pos, w, h);
            }

            node.position = new_pos;
        }
    }

    /// Move selected nodes by a delta.
    pub fn move_selected_nodes(&mut self, dx: f64, dy: f64) {
        let selected = self.selected_nodes.clone();
        let snap_enabled = self.snap_grid.enabled;
        let snap_grid = self.snap_grid.clone();

        for id in selected {
            if let Some(node) = self.get_node_mut(&id) {
                if node.draggable {
                    let new_pos = Position::new(node.position.x + dx, node.position.y + dy);
                    let final_pos = if snap_enabled {
                        snap_grid.snap(new_pos)
                    } else {
                        new_pos
                    };

                    // Apply extent constraints
                    let final_pos = if let Some(extent) = node.extent {
                        let w = node.width.unwrap_or(150.0);
                        let h = node.height.unwrap_or(40.0);
                        extent.clamp(final_pos, w, h)
                    } else {
                        final_pos
                    };

                    node.position = final_pos;
                }
            }
        }
    }

    /// Bring a node to front (increase z-index).
    pub fn bring_to_front(&mut self, id: &str) {
        self.max_z_index += 1;
        let new_z = self.max_z_index;
        if let Some(node) = self.get_node_mut(id) {
            node.z_index = new_z;
        }
    }

    /// Send a node to back (decrease z-index).
    pub fn send_to_back(&mut self, id: &str) {
        // Decrease all z-indices by 1, then set target to 0
        for node in &mut self.nodes {
            if node.id != id && node.z_index > 0 {
                node.z_index += 1;
            }
        }
        if let Some(node) = self.get_node_mut(id) {
            node.z_index = 0;
        }
    }

    /// Select a node.
    pub fn select_node(&mut self, id: &str, multi_select: bool) {
        // Check if node is selectable
        if let Some(node) = self.get_node(id) {
            if !node.selectable {
                return;
            }
        }

        if !multi_select {
            self.clear_selection();
        }
        if !self.selected_nodes.contains(&id.to_string()) {
            self.selected_nodes.push(id.to_string());
        }
        if let Some(node) = self.get_node_mut(id) {
            node.selected = true;
        }
    }

    /// Select an edge.
    pub fn select_edge(&mut self, id: &str, multi_select: bool) {
        // Check if edge is selectable
        if let Some(edge) = self.get_edge(id) {
            if !edge.selectable {
                return;
            }
        }

        if !multi_select {
            self.clear_selection();
        }
        if !self.selected_edges.contains(&id.to_string()) {
            self.selected_edges.push(id.to_string());
        }
        if let Some(edge) = self.edges.iter_mut().find(|e| e.id == id) {
            edge.selected = true;
        }
    }

    /// Select multiple nodes.
    pub fn select_nodes(&mut self, ids: &[&str], multi_select: bool) {
        if !multi_select {
            self.clear_selection();
        }
        for id in ids {
            if let Some(node) = self.get_node(id) {
                if node.selectable && !self.selected_nodes.contains(&id.to_string()) {
                    self.selected_nodes.push(id.to_string());
                }
            }
            if let Some(node) = self.get_node_mut(id) {
                node.selected = true;
            }
        }
    }

    /// Select nodes within a rectangle (box selection).
    pub fn select_in_rect(&mut self, rect: SelectionRect, multi_select: bool) {
        if !multi_select {
            self.clear_selection();
        }

        let node_ids: Vec<String> = self
            .nodes
            .iter()
            .filter(|n| n.selectable && rect.intersects_node(n))
            .map(|n| n.id.clone())
            .collect();

        for id in node_ids {
            if !self.selected_nodes.contains(&id) {
                self.selected_nodes.push(id.clone());
            }
            if let Some(node) = self.get_node_mut(&id) {
                node.selected = true;
            }
        }
    }

    /// Select all nodes and edges.
    pub fn select_all(&mut self) {
        for node in &mut self.nodes {
            if node.selectable {
                node.selected = true;
                if !self.selected_nodes.contains(&node.id) {
                    self.selected_nodes.push(node.id.clone());
                }
            }
        }
        for edge in &mut self.edges {
            if edge.selectable {
                edge.selected = true;
                if !self.selected_edges.contains(&edge.id) {
                    self.selected_edges.push(edge.id.clone());
                }
            }
        }
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        for node in &mut self.nodes {
            node.selected = false;
        }
        for edge in &mut self.edges {
            edge.selected = false;
        }
        self.selected_nodes.clear();
        self.selected_edges.clear();
    }

    /// Copy selected nodes and edges to clipboard.
    pub fn copy_selected(&mut self) {
        let selected_node_ids: Vec<String> = self.selected_nodes.clone();

        // Copy selected nodes
        let nodes: Vec<Node<T>> = self
            .nodes
            .iter()
            .filter(|n| selected_node_ids.contains(&n.id))
            .cloned()
            .collect();

        // Copy edges that connect copied nodes
        let edges: Vec<Edge> = self
            .edges
            .iter()
            .filter(|e| {
                selected_node_ids.contains(&e.source) && selected_node_ids.contains(&e.target)
            })
            .cloned()
            .collect();

        self.clipboard = ClipboardData { nodes, edges };
    }

    /// Cut selected nodes and edges (copy + delete).
    pub fn cut_selected(&mut self) -> (Vec<NodeId>, Vec<EdgeId>) {
        self.copy_selected();
        self.delete_selected()
    }

    /// Paste nodes and edges from clipboard.
    pub fn paste(&mut self, offset: Position) -> Vec<NodeId> {
        if self.clipboard.nodes.is_empty() {
            return Vec::new();
        }

        // Clone clipboard data to avoid borrow issues
        let clipboard_nodes = self.clipboard.nodes.clone();
        let clipboard_edges = self.clipboard.edges.clone();

        // Create mapping from old IDs to new IDs
        let mut id_map: HashMap<String, String> = HashMap::new();
        let mut new_node_ids: Vec<NodeId> = Vec::new();

        // Clear selection before pasting
        self.clear_selection();

        // Paste nodes with new IDs and offset positions
        for node in clipboard_nodes {
            let new_id = format!("{}-copy-{}", node.id, uuid::Uuid::new_v4());
            id_map.insert(node.id.clone(), new_id.clone());

            let mut new_node = node.clone();
            new_node.id = new_id.clone();
            new_node.position = Position::new(
                node.position.x + offset.x,
                node.position.y + offset.y,
            );
            new_node.selected = true;

            self.add_node(new_node);
            self.selected_nodes.push(new_id.clone());
            new_node_ids.push(new_id);
        }

        // Paste edges with updated node references
        for edge in clipboard_edges {
            if let (Some(new_source), Some(new_target)) =
                (id_map.get(&edge.source), id_map.get(&edge.target))
            {
                let new_id = format!("{}-copy-{}", edge.id, uuid::Uuid::new_v4());
                let mut new_edge = edge.clone();
                new_edge.id = new_id;
                new_edge.source = new_source.clone();
                new_edge.target = new_target.clone();
                self.add_edge(new_edge);
            }
        }

        new_node_ids
    }

    /// Check if clipboard has content.
    pub fn has_clipboard_content(&self) -> bool {
        !self.clipboard.nodes.is_empty()
    }

    /// Update the viewport.
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    /// Pan the viewport.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.viewport.x += dx;
        self.viewport.y += dy;
    }

    /// Zoom the viewport around a point.
    pub fn zoom(&mut self, delta: f64, center_x: f64, center_y: f64) {
        let old_zoom = self.viewport.zoom;
        let new_zoom = (old_zoom + delta).clamp(0.1, 4.0);

        // Adjust position to zoom around the center point
        self.viewport.x = center_x - (center_x - self.viewport.x) * new_zoom / old_zoom;
        self.viewport.y = center_y - (center_y - self.viewport.y) * new_zoom / old_zoom;
        self.viewport.zoom = new_zoom;
    }

    /// Set zoom level.
    pub fn set_zoom(&mut self, zoom: f64, center_x: f64, center_y: f64) {
        let old_zoom = self.viewport.zoom;
        let new_zoom = zoom.clamp(0.1, 4.0);

        self.viewport.x = center_x - (center_x - self.viewport.x) * new_zoom / old_zoom;
        self.viewport.y = center_y - (center_y - self.viewport.y) * new_zoom / old_zoom;
        self.viewport.zoom = new_zoom;
    }

    /// Zoom in by a fixed amount.
    pub fn zoom_in(&mut self, center_x: f64, center_y: f64) {
        self.zoom(0.2, center_x, center_y);
    }

    /// Zoom out by a fixed amount.
    pub fn zoom_out(&mut self, center_x: f64, center_y: f64) {
        self.zoom(-0.2, center_x, center_y);
    }

    /// Fit the view to show all nodes.
    pub fn fit_view(&mut self, padding: f64, container_width: f64, container_height: f64) {
        if self.nodes.is_empty() {
            return;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for node in &self.nodes {
            let (w, h) = self
                .node_dimensions
                .get(&node.id)
                .copied()
                .unwrap_or((node.width.unwrap_or(150.0), node.height.unwrap_or(40.0)));

            min_x = min_x.min(node.position.x);
            min_y = min_y.min(node.position.y);
            max_x = max_x.max(node.position.x + w);
            max_y = max_y.max(node.position.y + h);
        }

        let content_width = max_x - min_x + padding * 2.0;
        let content_height = max_y - min_y + padding * 2.0;

        let zoom_x = container_width / content_width;
        let zoom_y = container_height / content_height;
        let zoom = zoom_x.min(zoom_y).min(1.0).max(0.1);

        self.viewport.zoom = zoom;
        self.viewport.x = (container_width - content_width * zoom) / 2.0 - (min_x - padding) * zoom;
        self.viewport.y =
            (container_height - content_height * zoom) / 2.0 - (min_y - padding) * zoom;
    }

    /// Set snap grid configuration.
    pub fn set_snap_grid(&mut self, snap_grid: SnapGrid) {
        self.snap_grid = snap_grid;
    }

    /// Enable or disable snap to grid.
    pub fn set_snap_enabled(&mut self, enabled: bool) {
        self.snap_grid.enabled = enabled;
    }

    /// Validate a pending connection.
    pub fn validate_connection(&self, pending: &PendingConnection) -> ConnectionValidation {
        // Don't allow self-connections
        if pending.source == pending.target {
            return ConnectionValidation::invalid("Cannot connect a node to itself");
        }

        // Check if connection already exists
        let exists = self.edges.iter().any(|e| {
            e.source == pending.source
                && e.target == pending.target
                && e.source_handle == pending.source_handle
                && e.target_handle == pending.target_handle
        });

        if exists {
            return ConnectionValidation::invalid("Connection already exists");
        }

        ConnectionValidation::valid()
    }

    /// Start a new connection from a handle.
    pub fn start_connection(
        &mut self,
        node_id: NodeId,
        handle_position: crate::types::HandlePosition,
        position: Position,
    ) {
        self.connection = Some(Connection {
            source: node_id,
            source_handle: handle_position,
            source_handle_id: None,
            target_position: position,
        });
    }

    /// Start a new connection from a specific handle ID.
    pub fn start_connection_from_handle(
        &mut self,
        node_id: NodeId,
        handle_id: String,
        handle_position: crate::types::HandlePosition,
        position: Position,
    ) {
        self.connection = Some(Connection {
            source: node_id,
            source_handle: handle_position,
            source_handle_id: Some(handle_id),
            target_position: position,
        });
    }

    /// Update the connection target position.
    pub fn update_connection(&mut self, position: Position) {
        if let Some(conn) = &mut self.connection {
            conn.target_position = position;
        }
    }

    /// Cancel the current connection.
    pub fn cancel_connection(&mut self) {
        self.connection = None;
    }

    /// Complete a connection to a target handle.
    pub fn complete_connection(
        &mut self,
        target: NodeId,
        target_handle: crate::types::HandlePosition,
    ) -> Option<Edge> {
        self.complete_connection_to_handle(target, target_handle, None)
    }

    /// Complete a connection to a target handle with optional handle ID.
    pub fn complete_connection_to_handle(
        &mut self,
        target: NodeId,
        target_handle: crate::types::HandlePosition,
        target_handle_id: Option<String>,
    ) -> Option<Edge> {
        if let Some(conn) = self.connection.take() {
            // Don't connect a node to itself
            if conn.source == target {
                return None;
            }

            // Validate connection
            let pending = PendingConnection {
                source: conn.source.clone(),
                source_handle: conn.source_handle,
                target: target.clone(),
                target_handle,
            };

            if !self.validate_connection(&pending).is_valid {
                return None;
            }

            let mut edge = Edge::new(
                format!("e{}-{}", conn.source, target),
                conn.source.clone(),
                target,
            )
            .with_source_handle(conn.source_handle)
            .with_target_handle(target_handle)
            .with_type(self.default_edge_options.edge_type)
            .with_stroke(&self.default_edge_options.stroke)
            .with_stroke_width(self.default_edge_options.stroke_width)
            .with_animated(self.default_edge_options.animated);

            // Set handle IDs if available
            if let Some(src_handle_id) = conn.source_handle_id {
                edge = edge.with_source_handle_id(src_handle_id);
            }
            if let Some(tgt_handle_id) = target_handle_id {
                edge = edge.with_target_handle_id(tgt_handle_id);
            }

            self.add_edge(edge.clone());
            Some(edge)
        } else {
            None
        }
    }

    /// Get nodes sorted by z-index (for rendering).
    pub fn nodes_sorted_by_z_index(&self) -> Vec<&Node<T>> {
        let mut nodes: Vec<&Node<T>> = self.nodes.iter().collect();
        nodes.sort_by_key(|n| n.z_index);
        nodes
    }
}

/// Hook to use flow state.
pub fn use_flow<T: Clone + Default + PartialEq + 'static>(
    initial_nodes: Vec<Node<T>>,
    initial_edges: Vec<Edge>,
) -> Signal<FlowState<T>> {
    use_signal(|| FlowState::with_nodes_and_edges(initial_nodes, initial_edges))
}

/// Hook to handle flow events.
pub fn use_flow_events<F>(mut handler: F)
where
    F: FnMut(FlowEvent) + 'static,
{
    use_hook(move || {
        // This is a placeholder for event handling
        // In a real implementation, we'd set up a channel or callback system
        let _ = &mut handler;
    });
}

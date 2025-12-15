//! State management hooks for dioxus-flow.

use crate::types::{Connection, Edge, EdgeId, FlowEvent, Node, NodeId, Position, Viewport};
use dioxus::prelude::*;
use std::collections::HashMap;

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
        }
    }

    /// Create flow state with initial nodes and edges.
    pub fn with_nodes_and_edges(nodes: Vec<Node<T>>, edges: Vec<Edge>) -> Self {
        Self {
            nodes,
            edges,
            viewport: Viewport::default(),
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            connection: None,
            node_dimensions: HashMap::new(),
        }
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

    /// Add a node to the flow.
    pub fn add_node(&mut self, node: Node<T>) {
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

    /// Update a node's position.
    pub fn update_node_position(&mut self, id: &str, position: Position) {
        if let Some(node) = self.get_node_mut(id) {
            node.position = position;
        }
    }

    /// Select a node.
    pub fn select_node(&mut self, id: &str, multi_select: bool) {
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
        if let Some(conn) = self.connection.take() {
            // Don't connect a node to itself
            if conn.source == target {
                return None;
            }

            let edge = Edge::new(
                format!("e{}-{}", conn.source, target),
                conn.source.clone(),
                target,
            )
            .with_source_handle(conn.source_handle)
            .with_target_handle(target_handle);

            self.add_edge(edge.clone());
            Some(edge)
        } else {
            None
        }
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

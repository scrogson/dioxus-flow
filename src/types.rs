//! Core types for dioxus-flow.

use std::collections::HashMap;

/// Unique identifier for nodes and edges.
pub type NodeId = String;
pub type EdgeId = String;

/// Position in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Represents the viewport state (pan and zoom).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    pub fn new(x: f64, y: f64, zoom: f64) -> Self {
        Self { x, y, zoom }
    }

    /// Convert screen coordinates to flow coordinates.
    pub fn screen_to_flow(&self, screen_x: f64, screen_y: f64) -> Position {
        Position {
            x: (screen_x - self.x) / self.zoom,
            y: (screen_y - self.y) / self.zoom,
        }
    }

    /// Convert flow coordinates to screen coordinates.
    pub fn flow_to_screen(&self, flow_x: f64, flow_y: f64) -> Position {
        Position {
            x: flow_x * self.zoom + self.x,
            y: flow_y * self.zoom + self.y,
        }
    }
}

/// Handle position on a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HandlePosition {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

impl HandlePosition {
    /// Get the offset from the node center for this handle position.
    pub fn offset(&self, width: f64, height: f64) -> Position {
        match self {
            HandlePosition::Top => Position::new(width / 2.0, 0.0),
            HandlePosition::Right => Position::new(width, height / 2.0),
            HandlePosition::Bottom => Position::new(width / 2.0, height),
            HandlePosition::Left => Position::new(0.0, height / 2.0),
        }
    }
}

/// A node in the flow.
#[derive(Debug, Clone, PartialEq)]
pub struct Node<T = ()> {
    /// Unique identifier for the node.
    pub id: NodeId,
    /// Position of the node in flow coordinates.
    pub position: Position,
    /// Width of the node (optional, defaults to auto-sizing).
    pub width: Option<f64>,
    /// Height of the node (optional, defaults to auto-sizing).
    pub height: Option<f64>,
    /// Custom data associated with the node.
    pub data: T,
    /// Whether the node is selected.
    pub selected: bool,
    /// Whether the node is draggable.
    pub draggable: bool,
    /// Whether the node is connectable.
    pub connectable: bool,
    /// Node type for custom rendering.
    pub node_type: String,
    /// Additional CSS classes.
    pub class: String,
    /// Additional styles.
    pub style: HashMap<String, String>,
}

impl<T: Default> Node<T> {
    /// Create a new node with default data.
    pub fn new(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            width: None,
            height: None,
            data: T::default(),
            selected: false,
            draggable: true,
            connectable: true,
            node_type: "default".to_string(),
            class: String::new(),
            style: HashMap::new(),
        }
    }
}

impl<T> Node<T> {
    /// Create a new node with custom data.
    pub fn with_data(mut self, data: T) -> Self {
        self.data = data;
        self
    }

    /// Set the node type.
    pub fn with_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_type = node_type.into();
        self
    }

    /// Set whether the node is draggable.
    pub fn with_draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Set whether the node is connectable.
    pub fn with_connectable(mut self, connectable: bool) -> Self {
        self.connectable = connectable;
        self
    }

    /// Set the node dimensions.
    pub fn with_dimensions(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Add a CSS class.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Add a style.
    pub fn with_style(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.style.insert(key.into(), value.into());
        self
    }

    /// Get the center position of the node.
    pub fn center(&self) -> Position {
        let w = self.width.unwrap_or(150.0);
        let h = self.height.unwrap_or(40.0);
        Position::new(self.position.x + w / 2.0, self.position.y + h / 2.0)
    }

    /// Get handle position for a given handle position type.
    pub fn handle_position(&self, handle_pos: HandlePosition) -> Position {
        let w = self.width.unwrap_or(150.0);
        let h = self.height.unwrap_or(40.0);
        let offset = handle_pos.offset(w, h);
        Position::new(self.position.x + offset.x, self.position.y + offset.y)
    }
}

/// Edge type for different visual styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeType {
    #[default]
    Bezier,
    Straight,
    Step,
    SmoothStep,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Unique identifier for the edge.
    pub id: EdgeId,
    /// Source node ID.
    pub source: NodeId,
    /// Target node ID.
    pub target: NodeId,
    /// Source handle position.
    pub source_handle: HandlePosition,
    /// Target handle position.
    pub target_handle: HandlePosition,
    /// Edge type for rendering.
    pub edge_type: EdgeType,
    /// Whether the edge is animated.
    pub animated: bool,
    /// Whether the edge is selected.
    pub selected: bool,
    /// Edge label.
    pub label: Option<String>,
    /// Edge color.
    pub stroke: String,
    /// Edge width.
    pub stroke_width: f64,
    /// Additional CSS classes.
    pub class: String,
}

impl Edge {
    /// Create a new edge.
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            source_handle: HandlePosition::Bottom,
            target_handle: HandlePosition::Top,
            edge_type: EdgeType::default(),
            animated: false,
            selected: false,
            label: None,
            stroke: "#b1b1b7".to_string(),
            stroke_width: 2.0,
            class: String::new(),
        }
    }

    /// Set source handle position.
    pub fn with_source_handle(mut self, position: HandlePosition) -> Self {
        self.source_handle = position;
        self
    }

    /// Set target handle position.
    pub fn with_target_handle(mut self, position: HandlePosition) -> Self {
        self.target_handle = position;
        self
    }

    /// Set the edge type.
    pub fn with_type(mut self, edge_type: EdgeType) -> Self {
        self.edge_type = edge_type;
        self
    }

    /// Set whether the edge is animated.
    pub fn with_animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Set the edge label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the edge color.
    pub fn with_stroke(mut self, stroke: impl Into<String>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Set the edge width.
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Add a CSS class.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }
}

/// Connection state when dragging to create a new edge.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    /// Source node ID.
    pub source: NodeId,
    /// Source handle position.
    pub source_handle: HandlePosition,
    /// Current mouse position (target).
    pub target_position: Position,
}

/// Events emitted by the flow.
#[derive(Debug, Clone)]
pub enum FlowEvent {
    /// Node was clicked.
    NodeClick(NodeId),
    /// Node was double-clicked.
    NodeDoubleClick(NodeId),
    /// Node drag started.
    NodeDragStart(NodeId),
    /// Node was dragged.
    NodeDrag { id: NodeId, position: Position },
    /// Node drag ended.
    NodeDragEnd(NodeId),
    /// Edge was clicked.
    EdgeClick(EdgeId),
    /// Connection was started.
    ConnectStart {
        node_id: NodeId,
        handle_position: HandlePosition,
    },
    /// Connection was completed.
    Connect {
        source: NodeId,
        source_handle: HandlePosition,
        target: NodeId,
        target_handle: HandlePosition,
    },
    /// Pane was clicked.
    PaneClick(Position),
    /// Selection changed.
    SelectionChange {
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
    },
    /// Viewport changed.
    ViewportChange(Viewport),
}

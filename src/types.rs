//! Core types for dioxus-flow.

use std::collections::HashMap;

/// Unique identifier for nodes and edges.
pub type NodeId = String;
pub type EdgeId = String;
pub type HandleId = String;

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
    /// Get the offset from the node origin for this handle position.
    pub fn offset(&self, width: f64, height: f64) -> Position {
        match self {
            HandlePosition::Top => Position::new(width / 2.0, 0.0),
            HandlePosition::Right => Position::new(width, height / 2.0),
            HandlePosition::Bottom => Position::new(width / 2.0, height),
            HandlePosition::Left => Position::new(0.0, height / 2.0),
        }
    }

    /// Get offset with an index for multiple handles on the same side.
    /// index: 0-based index, count: total handles on this side
    pub fn offset_indexed(&self, width: f64, height: f64, index: usize, count: usize) -> Position {
        if count <= 1 {
            return self.offset(width, height);
        }

        let spacing = match self {
            HandlePosition::Top | HandlePosition::Bottom => width / (count + 1) as f64,
            HandlePosition::Left | HandlePosition::Right => height / (count + 1) as f64,
        };

        let pos = spacing * (index + 1) as f64;

        match self {
            HandlePosition::Top => Position::new(pos, 0.0),
            HandlePosition::Bottom => Position::new(pos, height),
            HandlePosition::Left => Position::new(0.0, pos),
            HandlePosition::Right => Position::new(width, pos),
        }
    }
}

/// Handle type - determines if this is an input or output connection point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HandleKind {
    /// Source/output handle - connections start from here.
    #[default]
    Source,
    /// Target/input handle - connections end here.
    Target,
}

/// A connection handle on a node.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeHandle {
    /// Unique identifier for this handle within the node.
    pub id: HandleId,
    /// Type of handle (source/output or target/input).
    pub kind: HandleKind,
    /// Position on the node (Top, Right, Bottom, Left).
    pub position: HandlePosition,
    /// Custom offset as percentage (0.0-1.0) along the edge, or None for auto.
    /// For Top/Bottom: 0.0 = left edge, 1.0 = right edge
    /// For Left/Right: 0.0 = top edge, 1.0 = bottom edge
    pub offset: Option<f64>,
    /// Whether this handle can accept connections.
    pub connectable: bool,
    /// Maximum number of connections (None = unlimited).
    pub max_connections: Option<usize>,
    /// Optional label for the handle.
    pub label: Option<String>,
}

impl NodeHandle {
    /// Create a new source (output) handle.
    pub fn source(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: HandleKind::Source,
            position: HandlePosition::Right,
            offset: None,
            connectable: true,
            max_connections: None,
            label: None,
        }
    }

    /// Create a new target (input) handle.
    pub fn target(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: HandleKind::Target,
            position: HandlePosition::Left,
            offset: None,
            connectable: true,
            max_connections: None,
            label: None,
        }
    }

    /// Set the position.
    pub fn with_position(mut self, position: HandlePosition) -> Self {
        self.position = position;
        self
    }

    /// Set custom offset (0.0-1.0).
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = Some(offset.clamp(0.0, 1.0));
        self
    }

    /// Set maximum connections.
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// Set a label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Calculate the absolute position of this handle on a node.
    pub fn absolute_position(&self, node_pos: Position, width: f64, height: f64) -> Position {
        let offset = if let Some(pct) = self.offset {
            // Custom percentage offset
            match self.position {
                HandlePosition::Top => Position::new(width * pct, 0.0),
                HandlePosition::Bottom => Position::new(width * pct, height),
                HandlePosition::Left => Position::new(0.0, height * pct),
                HandlePosition::Right => Position::new(width, height * pct),
            }
        } else {
            // Default center position
            self.position.offset(width, height)
        };

        Position::new(node_pos.x + offset.x, node_pos.y + offset.y)
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
    /// Whether the node is selectable.
    pub selectable: bool,
    /// Whether the node is draggable.
    pub draggable: bool,
    /// Whether the node is deletable.
    pub deletable: bool,
    /// Whether the node is connectable (legacy, use handles for fine control).
    pub connectable: bool,
    /// Connection handles on this node.
    pub handles: Vec<NodeHandle>,
    /// Node type for custom rendering.
    pub node_type: String,
    /// Z-index for layering (higher = on top).
    pub z_index: i32,
    /// Additional CSS classes.
    pub class: String,
    /// Additional styles.
    pub style: HashMap<String, String>,
    /// Movement extent/bounds (min_x, min_y, max_x, max_y). None = no bounds.
    pub extent: Option<NodeExtent>,
}

impl<T: Default> Node<T> {
    /// Create a new node with default data and default handles (top target, bottom source).
    pub fn new(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            width: None,
            height: None,
            data: T::default(),
            selected: false,
            selectable: true,
            draggable: true,
            deletable: true,
            connectable: true,
            handles: vec![
                NodeHandle::target("target").with_position(HandlePosition::Top),
                NodeHandle::source("source").with_position(HandlePosition::Bottom),
            ],
            node_type: "default".to_string(),
            z_index: 0,
            class: String::new(),
            style: HashMap::new(),
            extent: None,
        }
    }

    /// Create a new node with no handles.
    pub fn new_without_handles(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            position: Position::new(x, y),
            width: None,
            height: None,
            data: T::default(),
            selected: false,
            selectable: true,
            draggable: true,
            deletable: true,
            connectable: false,
            handles: Vec::new(),
            node_type: "default".to_string(),
            z_index: 0,
            class: String::new(),
            style: HashMap::new(),
            extent: None,
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

    /// Set whether the node is selectable.
    pub fn with_selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Set whether the node is deletable.
    pub fn with_deletable(mut self, deletable: bool) -> Self {
        self.deletable = deletable;
        self
    }

    /// Set the z-index for layering.
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Set the movement extent/bounds.
    pub fn with_extent(mut self, extent: NodeExtent) -> Self {
        self.extent = Some(extent);
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

    /// Set custom handles (replaces default handles).
    pub fn with_handles(mut self, handles: Vec<NodeHandle>) -> Self {
        self.handles = handles;
        self.connectable = !self.handles.is_empty();
        self
    }

    /// Add a single handle.
    pub fn with_handle(mut self, handle: NodeHandle) -> Self {
        self.handles.push(handle);
        self.connectable = true;
        self
    }

    /// Add multiple input handles on the left side.
    pub fn with_inputs(mut self, labels: &[&str]) -> Self {
        let count = labels.len();
        for (i, label) in labels.iter().enumerate() {
            let offset = (i + 1) as f64 / (count + 1) as f64;
            self.handles.push(
                NodeHandle::target(format!("input-{}", i))
                    .with_position(HandlePosition::Left)
                    .with_offset(offset)
                    .with_label(label.to_string()),
            );
        }
        self.connectable = true;
        self
    }

    /// Add multiple output handles on the right side.
    pub fn with_outputs(mut self, labels: &[&str]) -> Self {
        let count = labels.len();
        for (i, label) in labels.iter().enumerate() {
            let offset = (i + 1) as f64 / (count + 1) as f64;
            self.handles.push(
                NodeHandle::source(format!("output-{}", i))
                    .with_position(HandlePosition::Right)
                    .with_offset(offset)
                    .with_label(label.to_string()),
            );
        }
        self.connectable = true;
        self
    }

    /// Get a handle by ID.
    pub fn get_handle(&self, handle_id: &str) -> Option<&NodeHandle> {
        self.handles.iter().find(|h| h.id == handle_id)
    }

    /// Get all source (output) handles.
    pub fn source_handles(&self) -> impl Iterator<Item = &NodeHandle> {
        self.handles.iter().filter(|h| h.kind == HandleKind::Source)
    }

    /// Get all target (input) handles.
    pub fn target_handles(&self) -> impl Iterator<Item = &NodeHandle> {
        self.handles.iter().filter(|h| h.kind == HandleKind::Target)
    }

    /// Get the center position of the node.
    pub fn center(&self) -> Position {
        let w = self.width.unwrap_or(150.0);
        let h = self.height.unwrap_or(40.0);
        Position::new(self.position.x + w / 2.0, self.position.y + h / 2.0)
    }

    /// Get handle position for a given handle position type (legacy).
    pub fn handle_position(&self, handle_pos: HandlePosition) -> Position {
        let w = self.width.unwrap_or(150.0);
        let h = self.height.unwrap_or(40.0);
        let offset = handle_pos.offset(w, h);
        Position::new(self.position.x + offset.x, self.position.y + offset.y)
    }

    /// Get handle position by handle ID.
    pub fn handle_position_by_id(&self, handle_id: &str) -> Option<Position> {
        let w = self.width.unwrap_or(150.0);
        let h = self.height.unwrap_or(40.0);
        self.get_handle(handle_id)
            .map(|handle| handle.absolute_position(self.position, w, h))
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
    /// Source handle position (legacy, use source_handle_id for multiple handles).
    pub source_handle: HandlePosition,
    /// Target handle position (legacy, use target_handle_id for multiple handles).
    pub target_handle: HandlePosition,
    /// Source handle ID (takes precedence over source_handle if set).
    pub source_handle_id: Option<HandleId>,
    /// Target handle ID (takes precedence over target_handle if set).
    pub target_handle_id: Option<HandleId>,
    /// Edge type for rendering.
    pub edge_type: EdgeType,
    /// Whether the edge is animated.
    pub animated: bool,
    /// Whether the edge is selected.
    pub selected: bool,
    /// Whether the edge is selectable.
    pub selectable: bool,
    /// Whether the edge is deletable.
    pub deletable: bool,
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
    /// Create a new edge using default handles.
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            source_handle: HandlePosition::Bottom,
            target_handle: HandlePosition::Top,
            source_handle_id: None,
            target_handle_id: None,
            edge_type: EdgeType::default(),
            animated: false,
            selected: false,
            selectable: true,
            deletable: true,
            label: None,
            stroke: "#b1b1b7".to_string(),
            stroke_width: 2.0,
            class: String::new(),
        }
    }

    /// Create a new edge connecting specific handles by ID.
    pub fn new_with_handles(
        id: impl Into<String>,
        source: impl Into<String>,
        source_handle: impl Into<String>,
        target: impl Into<String>,
        target_handle: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            source_handle: HandlePosition::Right, // Fallback
            target_handle: HandlePosition::Left,  // Fallback
            source_handle_id: Some(source_handle.into()),
            target_handle_id: Some(target_handle.into()),
            edge_type: EdgeType::default(),
            animated: false,
            selected: false,
            selectable: true,
            deletable: true,
            label: None,
            stroke: "#b1b1b7".to_string(),
            stroke_width: 2.0,
            class: String::new(),
        }
    }

    /// Set whether the edge is selectable.
    pub fn with_selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Set whether the edge is deletable.
    pub fn with_deletable(mut self, deletable: bool) -> Self {
        self.deletable = deletable;
        self
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

    /// Set source handle by ID.
    pub fn with_source_handle_id(mut self, handle_id: impl Into<String>) -> Self {
        self.source_handle_id = Some(handle_id.into());
        self
    }

    /// Set target handle by ID.
    pub fn with_target_handle_id(mut self, handle_id: impl Into<String>) -> Self {
        self.target_handle_id = Some(handle_id.into());
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
    /// Source handle position (legacy).
    pub source_handle: HandlePosition,
    /// Source handle ID (if using multiple handles).
    pub source_handle_id: Option<HandleId>,
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
    /// Nodes were deleted.
    NodesDelete(Vec<NodeId>),
    /// Edges were deleted.
    EdgesDelete(Vec<EdgeId>),
}

/// Snap grid configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapGrid {
    /// Whether snap to grid is enabled.
    pub enabled: bool,
    /// Grid cell size in pixels.
    pub size: f64,
}

impl Default for SnapGrid {
    fn default() -> Self {
        Self {
            enabled: false,
            size: 15.0,
        }
    }
}

impl SnapGrid {
    /// Create a new snap grid.
    pub fn new(size: f64) -> Self {
        Self {
            enabled: true,
            size,
        }
    }

    /// Snap a position to the grid.
    pub fn snap(&self, position: Position) -> Position {
        if !self.enabled {
            return position;
        }
        Position {
            x: (position.x / self.size).round() * self.size,
            y: (position.y / self.size).round() * self.size,
        }
    }
}

/// Connection validation result.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionValidation {
    /// Whether the connection is valid.
    pub is_valid: bool,
    /// Optional message explaining why invalid.
    pub message: Option<String>,
}

impl ConnectionValidation {
    /// Create a valid connection result.
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            message: None,
        }
    }

    /// Create an invalid connection result.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            message: Some(message.into()),
        }
    }
}

/// Pending connection info for validation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingConnection {
    /// Source node ID.
    pub source: NodeId,
    /// Source handle position.
    pub source_handle: HandlePosition,
    /// Target node ID.
    pub target: NodeId,
    /// Target handle position.
    pub target_handle: HandlePosition,
}

/// Edge marker (arrow) type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerType {
    #[default]
    Arrow,
    ArrowClosed,
    None,
}

/// Marker configuration for edge ends.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeMarker {
    /// Marker type.
    pub marker_type: MarkerType,
    /// Marker color (defaults to edge stroke color).
    pub color: Option<String>,
    /// Marker width.
    pub width: f64,
    /// Marker height.
    pub height: f64,
}

impl Default for EdgeMarker {
    fn default() -> Self {
        Self {
            marker_type: MarkerType::Arrow,
            color: None,
            width: 12.0,
            height: 12.0,
        }
    }
}

/// Selection rectangle for multi-select.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SelectionRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl SelectionRect {
    /// Check if a point is inside the rectangle.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Check if another rectangle intersects with this one.
    pub fn intersects(&self, other: &SelectionRect) -> bool {
        !(other.x > self.x + self.width
            || other.x + other.width < self.x
            || other.y > self.y + self.height
            || other.y + other.height < self.y)
    }

    /// Check if a node intersects with this rectangle.
    pub fn intersects_node<T>(&self, node: &Node<T>) -> bool {
        let node_rect = SelectionRect {
            x: node.position.x,
            y: node.position.y,
            width: node.width.unwrap_or(150.0),
            height: node.height.unwrap_or(40.0),
        };
        self.intersects(&node_rect)
    }
}

/// Keyboard modifiers state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyboardModifiers {
    /// Create from a keyboard event.
    pub fn from_keyboard_event(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        Self {
            shift,
            ctrl,
            alt,
            meta,
        }
    }
}

/// Node extent/bounds for constraining movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeExtent {
    /// Minimum X position.
    pub min_x: f64,
    /// Minimum Y position.
    pub min_y: f64,
    /// Maximum X position.
    pub max_x: f64,
    /// Maximum Y position.
    pub max_y: f64,
}

impl NodeExtent {
    /// Create a new extent.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Create an extent that constrains to a parent node.
    pub fn parent(parent_width: f64, parent_height: f64) -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            max_x: parent_width,
            max_y: parent_height,
        }
    }

    /// Clamp a position to this extent.
    pub fn clamp(&self, position: Position, node_width: f64, node_height: f64) -> Position {
        Position {
            x: position.x.clamp(self.min_x, self.max_x - node_width),
            y: position.y.clamp(self.min_y, self.max_y - node_height),
        }
    }
}

/// Interactivity configuration for the flow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractivityConfig {
    /// Whether nodes can be dragged.
    pub nodes_draggable: bool,
    /// Whether nodes can be connected.
    pub nodes_connectable: bool,
    /// Whether nodes are selectable.
    pub nodes_selectable: bool,
    /// Whether edges are selectable.
    pub edges_selectable: bool,
    /// Whether elements can be deleted with keyboard.
    pub elements_deletable: bool,
    /// Whether panning is enabled.
    pub pan_on_drag: bool,
    /// Whether to pan on scroll (vs zoom).
    pub pan_on_scroll: bool,
    /// Whether zoom on scroll is enabled.
    pub zoom_on_scroll: bool,
    /// Whether zoom on pinch is enabled.
    pub zoom_on_pinch: bool,
    /// Whether zoom on double-click is enabled.
    pub zoom_on_double_click: bool,
    /// Whether to select on drag (box selection).
    pub selection_on_drag: bool,
}

impl Default for InteractivityConfig {
    fn default() -> Self {
        Self {
            nodes_draggable: true,
            nodes_connectable: true,
            nodes_selectable: true,
            edges_selectable: true,
            elements_deletable: true,
            pan_on_drag: true,
            pan_on_scroll: false,
            zoom_on_scroll: true,
            zoom_on_pinch: true,
            zoom_on_double_click: true,
            selection_on_drag: false,
        }
    }
}

/// Default edge options applied to new edges.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEdgeOptions {
    /// Default edge type.
    pub edge_type: EdgeType,
    /// Default stroke color.
    pub stroke: String,
    /// Default stroke width.
    pub stroke_width: f64,
    /// Default animated state.
    pub animated: bool,
}

impl Default for DefaultEdgeOptions {
    fn default() -> Self {
        Self {
            edge_type: EdgeType::Bezier,
            stroke: "#b1b1b7".to_string(),
            stroke_width: 2.0,
            animated: false,
        }
    }
}

/// Copy/paste clipboard data.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipboardData<T: Clone> {
    /// Copied nodes.
    pub nodes: Vec<Node<T>>,
    /// Copied edges (only those connecting copied nodes).
    pub edges: Vec<Edge>,
}

impl<T: Clone> Default for ClipboardData<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

/// Default node dimensions.
pub const DEFAULT_NODE_WIDTH: f64 = 150.0;
pub const DEFAULT_NODE_HEIGHT: f64 = 40.0;

//! Selection box component for multi-selecting nodes.

use crate::types::Position;
use dioxus::prelude::*;

/// Selection box state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectionBoxState {
    /// Whether selection is active.
    pub active: bool,
    /// Start position of the selection.
    pub start: Position,
    /// Current end position.
    pub end: Position,
}

impl SelectionBoxState {
    /// Create a new selection box state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new selection.
    pub fn start(&mut self, position: Position) {
        self.active = true;
        self.start = position;
        self.end = position;
    }

    /// Update the selection end position.
    pub fn update(&mut self, position: Position) {
        if self.active {
            self.end = position;
        }
    }

    /// End the selection and return the bounding box.
    pub fn finish(&mut self) -> Option<SelectionRect> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some(self.get_rect())
    }

    /// Cancel the selection.
    pub fn cancel(&mut self) {
        self.active = false;
    }

    /// Get the normalized selection rectangle.
    pub fn get_rect(&self) -> SelectionRect {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);

        SelectionRect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    /// Check if a point is within the selection.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let rect = self.get_rect();
        x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
    }

    /// Check if a rectangle intersects with the selection.
    pub fn intersects(&self, x: f64, y: f64, width: f64, height: f64) -> bool {
        let rect = self.get_rect();
        !(x > rect.x + rect.width
            || x + width < rect.x
            || y > rect.y + rect.height
            || y + height < rect.y)
    }
}

/// A normalized selection rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SelectionRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Selection box component props.
#[derive(Props, Clone, PartialEq)]
pub struct SelectionBoxProps {
    /// Selection box state.
    pub state: SelectionBoxState,
    /// Color of the selection box.
    #[props(default = "rgba(0, 89, 220, 0.08)".to_string())]
    pub background_color: String,
    /// Border color.
    #[props(default = "rgba(0, 89, 220, 0.8)".to_string())]
    pub border_color: String,
}

/// Visual selection box component.
#[component]
pub fn SelectionBox(props: SelectionBoxProps) -> Element {
    if !props.state.active {
        return rsx! {};
    }

    let rect = props.state.get_rect();

    // Don't render if too small
    if rect.width < 5.0 && rect.height < 5.0 {
        return rsx! {};
    }

    rsx! {
        div {
            class: "dioxus-flow-selection-box",
            style: "position: absolute; left: {rect.x}px; top: {rect.y}px; width: {rect.width}px; height: {rect.height}px; background: {props.background_color}; border: 1px solid {props.border_color}; pointer-events: none;",
        }
    }
}

/// CSS styles for the selection box.
pub const SELECTION_BOX_STYLES: &str = r#"
.dioxus-flow-selection-box {
    z-index: 10;
    border-radius: 2px;
}
"#;

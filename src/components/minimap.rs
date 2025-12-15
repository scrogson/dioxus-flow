//! Minimap component for flow overview navigation.

use crate::hooks::FlowState;
use crate::types::Viewport;
use dioxus::prelude::*;

/// Minimap position on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MiniMapPosition {
    TopLeft,
    TopRight,
    #[default]
    BottomRight,
    BottomLeft,
}

/// Minimap component props.
#[derive(Props, Clone, PartialEq)]
pub struct MiniMapProps<T: Clone + PartialEq + 'static> {
    /// Flow state to visualize.
    pub state: Signal<FlowState<T>>,
    /// Width of the minimap.
    #[props(default = 200.0)]
    pub width: f64,
    /// Height of the minimap.
    #[props(default = 150.0)]
    pub height: f64,
    /// Position of the minimap.
    #[props(default)]
    pub position: MiniMapPosition,
    /// Node color (CSS color string).
    #[props(default = "#e2e2e2".to_string())]
    pub node_color: String,
    /// Node stroke color.
    #[props(default = "#b1b1b7".to_string())]
    pub node_stroke_color: String,
    /// Mask color for the non-visible area.
    #[props(default = "rgba(240, 240, 240, 0.6)".to_string())]
    pub mask_color: String,
    /// Whether the minimap is pannable.
    #[props(default = true)]
    pub pannable: bool,
    /// Whether the minimap is zoomable.
    #[props(default = true)]
    pub zoomable: bool,
    /// Callback when viewport changes via minimap.
    #[props(default)]
    pub on_viewport_change: Option<EventHandler<Viewport>>,
}

/// Minimap component showing an overview of the flow.
#[component]
pub fn MiniMap<T: Clone + Default + PartialEq + 'static>(props: MiniMapProps<T>) -> Element {
    let mut state = props.state;
    let nodes = state.read().nodes.clone();
    let viewport = state.read().viewport;

    // Calculate bounds of all nodes
    let (min_x, min_y, max_x, max_y) = calculate_bounds(&nodes);
    let content_width = (max_x - min_x).max(100.0);
    let content_height = (max_y - min_y).max(100.0);

    // Add padding
    let padding = 50.0;
    let padded_min_x = min_x - padding;
    let padded_min_y = min_y - padding;
    let padded_width = content_width + padding * 2.0;
    let padded_height = content_height + padding * 2.0;

    // Calculate scale to fit content in minimap
    let scale_x = props.width / padded_width;
    let scale_y = props.height / padded_height;
    let scale = scale_x.min(scale_y);

    // Calculate viewport rectangle in minimap coordinates
    // This represents what's currently visible in the main flow
    let vp_x = (-viewport.x / viewport.zoom - padded_min_x) * scale;
    let vp_y = (-viewport.y / viewport.zoom - padded_min_y) * scale;
    // Assuming container is roughly 800x600 for now (would need actual dimensions)
    let vp_width = (800.0 / viewport.zoom) * scale;
    let vp_height = (600.0 / viewport.zoom) * scale;

    let position_style = match props.position {
        MiniMapPosition::TopLeft => "top: 10px; left: 10px;",
        MiniMapPosition::TopRight => "top: 10px; right: 10px;",
        MiniMapPosition::BottomRight => "bottom: 10px; right: 10px;",
        MiniMapPosition::BottomLeft => "bottom: 10px; left: 10px;",
    };

    let width = props.width;
    let height = props.height;
    let pannable = props.pannable;
    let on_viewport_change = props.on_viewport_change.clone();

    let on_click = move |evt: MouseEvent| {
        if !pannable {
            return;
        }

        let coords = evt.client_coordinates();
        // Convert click position to flow coordinates
        // This is a simplified version - would need element bounds for accuracy
        let click_x = coords.x;
        let click_y = coords.y;

        // Calculate new viewport center
        let flow_x = (click_x / scale) + padded_min_x;
        let flow_y = (click_y / scale) + padded_min_y;

        let new_viewport = Viewport {
            x: -flow_x * viewport.zoom + 400.0, // Assuming 800px container
            y: -flow_y * viewport.zoom + 300.0, // Assuming 600px container
            zoom: viewport.zoom,
        };

        state.write().set_viewport(new_viewport);
        if let Some(handler) = &on_viewport_change {
            handler.call(new_viewport);
        }
    };

    rsx! {
        div {
            class: "dioxus-flow-minimap",
            style: "position: absolute; {position_style} width: {width}px; height: {height}px; background: white; border: 1px solid #ddd; border-radius: 4px; overflow: hidden; box-shadow: 0 2px 6px rgba(0,0,0,0.1);",

            svg {
                width: "{width}",
                height: "{height}",
                onclick: on_click,

                // Render nodes as simple rectangles
                for node in nodes.iter() {
                    rect {
                        x: "{(node.position.x - padded_min_x) * scale}",
                        y: "{(node.position.y - padded_min_y) * scale}",
                        width: "{node.width.unwrap_or(150.0) * scale}",
                        height: "{node.height.unwrap_or(40.0) * scale}",
                        fill: "{props.node_color}",
                        stroke: "{props.node_stroke_color}",
                        stroke_width: "1",
                        rx: "2",
                    }
                }

                // Viewport indicator (mask around visible area)
                defs {
                    mask {
                        id: "minimap-mask",
                        rect {
                            x: "0",
                            y: "0",
                            width: "{width}",
                            height: "{height}",
                            fill: "white",
                        }
                        rect {
                            x: "{vp_x}",
                            y: "{vp_y}",
                            width: "{vp_width}",
                            height: "{vp_height}",
                            fill: "black",
                        }
                    }
                }

                // Mask overlay
                rect {
                    x: "0",
                    y: "0",
                    width: "{width}",
                    height: "{height}",
                    fill: "{props.mask_color}",
                    mask: "url(#minimap-mask)",
                }

                // Viewport rectangle border
                rect {
                    x: "{vp_x}",
                    y: "{vp_y}",
                    width: "{vp_width}",
                    height: "{vp_height}",
                    fill: "none",
                    stroke: "#1a192b",
                    stroke_width: "1",
                    class: "dioxus-flow-minimap-viewport",
                }
            }
        }
    }
}

/// Calculate the bounding box of all nodes.
fn calculate_bounds<T: Clone + PartialEq>(nodes: &[crate::types::Node<T>]) -> (f64, f64, f64, f64) {
    if nodes.is_empty() {
        return (0.0, 0.0, 100.0, 100.0);
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        let w = node.width.unwrap_or(150.0);
        let h = node.height.unwrap_or(40.0);

        min_x = min_x.min(node.position.x);
        min_y = min_y.min(node.position.y);
        max_x = max_x.max(node.position.x + w);
        max_y = max_y.max(node.position.y + h);
    }

    (min_x, min_y, max_x, max_y)
}

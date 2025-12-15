//! Controls component for zoom and fit operations.

use crate::hooks::FlowState;
use crate::types::Viewport;
use dioxus::prelude::*;

/// Controls position on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlsPosition {
    TopLeft,
    TopRight,
    #[default]
    BottomLeft,
    BottomRight,
}

/// Controls component props.
#[derive(Props, Clone, PartialEq)]
pub struct ControlsProps<T: Clone + PartialEq + 'static> {
    /// Flow state to control.
    pub state: Signal<FlowState<T>>,
    /// Position of the controls.
    #[props(default)]
    pub position: ControlsPosition,
    /// Whether to show zoom in button.
    #[props(default = true)]
    pub show_zoom_in: bool,
    /// Whether to show zoom out button.
    #[props(default = true)]
    pub show_zoom_out: bool,
    /// Whether to show fit view button.
    #[props(default = true)]
    pub show_fit_view: bool,
    /// Whether to show lock/interactive toggle button.
    #[props(default = false)]
    pub show_interactive: bool,
    /// Minimum zoom level.
    #[props(default = 0.1)]
    pub min_zoom: f64,
    /// Maximum zoom level.
    #[props(default = 4.0)]
    pub max_zoom: f64,
    /// Zoom step amount.
    #[props(default = 0.2)]
    pub zoom_step: f64,
    /// Callback when viewport changes.
    #[props(default)]
    pub on_viewport_change: Option<EventHandler<Viewport>>,
    /// Callback when interactive state changes.
    #[props(default)]
    pub on_interactive_change: Option<EventHandler<bool>>,
}

/// Controls component for zoom and navigation.
#[component]
pub fn Controls<T: Clone + Default + PartialEq + 'static>(props: ControlsProps<T>) -> Element {
    let mut state = props.state;
    let mut is_interactive = use_signal(|| true);

    let position_style = match props.position {
        ControlsPosition::TopLeft => "top: 10px; left: 10px;",
        ControlsPosition::TopRight => "top: 10px; right: 10px;",
        ControlsPosition::BottomLeft => "bottom: 10px; left: 10px;",
        ControlsPosition::BottomRight => "bottom: 10px; right: 10px;",
    };

    let min_zoom = props.min_zoom;
    let max_zoom = props.max_zoom;
    let zoom_step = props.zoom_step;
    let on_viewport_change = props.on_viewport_change.clone();
    let on_interactive_change = props.on_interactive_change.clone();

    let zoom_in = move |_| {
        let mut s = state.write();
        let old_zoom = s.viewport.zoom;
        let new_zoom = (old_zoom + zoom_step).min(max_zoom);

        // Zoom around center (assuming 800x600 container)
        let center_x = 400.0;
        let center_y = 300.0;
        s.viewport.x = center_x - (center_x - s.viewport.x) * new_zoom / old_zoom;
        s.viewport.y = center_y - (center_y - s.viewport.y) * new_zoom / old_zoom;
        s.viewport.zoom = new_zoom;

        if let Some(handler) = &on_viewport_change {
            handler.call(s.viewport);
        }
    };

    let zoom_out = move |_| {
        let mut s = state.write();
        let old_zoom = s.viewport.zoom;
        let new_zoom = (old_zoom - zoom_step).max(min_zoom);

        let center_x = 400.0;
        let center_y = 300.0;
        s.viewport.x = center_x - (center_x - s.viewport.x) * new_zoom / old_zoom;
        s.viewport.y = center_y - (center_y - s.viewport.y) * new_zoom / old_zoom;
        s.viewport.zoom = new_zoom;

        if let Some(handler) = &on_viewport_change {
            handler.call(s.viewport);
        }
    };

    let fit_view = move |_| {
        // Reset to default view - in a real implementation would calculate bounds
        state.write().set_viewport(Viewport::default());
        if let Some(handler) = &on_viewport_change {
            handler.call(Viewport::default());
        }
    };

    let toggle_interactive = move |_| {
        let new_state = !*is_interactive.read();
        is_interactive.set(new_state);
        if let Some(handler) = &on_interactive_change {
            handler.call(new_state);
        }
    };

    rsx! {
        div {
            class: "dioxus-flow-controls",
            style: "position: absolute; {position_style} display: flex; flex-direction: column; gap: 4px;",

            if props.show_zoom_in {
                button {
                    class: "dioxus-flow-controls-button",
                    onclick: zoom_in,
                    title: "Zoom In",
                    // Plus icon
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        line { x1: "12", y1: "5", x2: "12", y2: "19" }
                        line { x1: "5", y1: "12", x2: "19", y2: "12" }
                    }
                }
            }

            if props.show_zoom_out {
                button {
                    class: "dioxus-flow-controls-button",
                    onclick: zoom_out,
                    title: "Zoom Out",
                    // Minus icon
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        line { x1: "5", y1: "12", x2: "19", y2: "12" }
                    }
                }
            }

            if props.show_fit_view {
                button {
                    class: "dioxus-flow-controls-button",
                    onclick: fit_view,
                    title: "Fit View",
                    // Maximize icon
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        polyline { points: "15 3 21 3 21 9" }
                        polyline { points: "9 21 3 21 3 15" }
                        line { x1: "21", y1: "3", x2: "14", y2: "10" }
                        line { x1: "3", y1: "21", x2: "10", y2: "14" }
                    }
                }
            }

            if props.show_interactive {
                button {
                    class: "dioxus-flow-controls-button",
                    class: if !*is_interactive.read() { "dioxus-flow-controls-button-active" } else { "" },
                    onclick: toggle_interactive,
                    title: if *is_interactive.read() { "Lock" } else { "Unlock" },
                    // Lock icon
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        if *is_interactive.read() {
                            // Unlocked
                            rect { x: "3", y: "11", width: "18", height: "11", rx: "2", ry: "2" }
                            path { d: "M7 11V7a5 5 0 0 1 9.9-1" }
                        } else {
                            // Locked
                            rect { x: "3", y: "11", width: "18", height: "11", rx: "2", ry: "2" }
                            path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                        }
                    }
                }
            }
        }
    }
}

/// CSS styles for the controls component.
pub const CONTROLS_STYLES: &str = r#"
.dioxus-flow-controls {
    z-index: 5;
}

.dioxus-flow-controls-button {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: white;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    color: #333;
    transition: all 0.2s;
}

.dioxus-flow-controls-button:hover {
    background: #f5f5f5;
    border-color: #bbb;
}

.dioxus-flow-controls-button:active {
    background: #eee;
}

.dioxus-flow-controls-button-active {
    background: #1a192b;
    color: white;
    border-color: #1a192b;
}

.dioxus-flow-controls-button-active:hover {
    background: #2a293b;
}
"#;

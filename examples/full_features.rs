//! Full features example demonstrating all dioxus-flow capabilities.
//!
//! Features demonstrated:
//! - Minimap for overview navigation
//! - Controls for zoom/fit operations
//! - Background patterns
//! - Multiple edge types
//! - Custom node styling
//! - Snap to grid
//! - Keyboard shortcuts

use dioxus::prelude::*;
use dioxus_flow::components::flow::FLOW_STYLES;
use dioxus_flow::components::controls::CONTROLS_STYLES;
use dioxus_flow::hooks::FlowState;
use dioxus_flow::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq, Default)]
struct TaskData {
    title: String,
    description: String,
    status: TaskStatus,
}

#[derive(Clone, PartialEq, Default)]
enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    fn class(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "task-todo",
            TaskStatus::InProgress => "task-progress",
            TaskStatus::Done => "task-done",
        }
    }
}

impl TaskData {
    fn new(title: &str, description: &str, status: TaskStatus) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            status,
        }
    }
}

#[component]
fn App() -> Element {
    // Create a sample task flow
    let initial_nodes = vec![
        Node::new("start", 50.0, 250.0)
            .with_label("Start")
            .with_data(TaskData::new("Start", "Project kickoff", TaskStatus::Done))
            .with_type("task-done")
            .with_dimensions(140.0, 44.0),
        Node::new("design", 250.0, 100.0)
            .with_label("Design")
            .with_data(TaskData::new("Design", "Create mockups", TaskStatus::Done))
            .with_type("task-done")
            .with_dimensions(140.0, 44.0),
        Node::new("research", 250.0, 400.0)
            .with_label("Research")
            .with_data(TaskData::new("Research", "Market analysis", TaskStatus::Done))
            .with_type("task-done")
            .with_dimensions(140.0, 44.0),
        Node::new("implement", 500.0, 200.0)
            .with_label("Implement")
            .with_data(TaskData::new("Implement", "Build features", TaskStatus::InProgress))
            .with_type("task-progress")
            .with_dimensions(140.0, 44.0),
        Node::new("test", 500.0, 350.0)
            .with_label("Test")
            .with_data(TaskData::new("Test", "QA testing", TaskStatus::Todo))
            .with_type("task-todo")
            .with_dimensions(140.0, 44.0),
        Node::new("review", 750.0, 250.0)
            .with_label("Review")
            .with_data(TaskData::new("Review", "Code review", TaskStatus::Todo))
            .with_type("task-todo")
            .with_dimensions(140.0, 44.0),
        Node::new("deploy", 950.0, 250.0)
            .with_label("Deploy")
            .with_data(TaskData::new("Deploy", "Ship to production", TaskStatus::Todo))
            .with_type("task-todo")
            .with_dimensions(140.0, 44.0),
    ];

    let initial_edges = vec![
        // From start
        Edge::new("e-start-design", "start", "design")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::SmoothStep),
        Edge::new("e-start-research", "start", "research")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::SmoothStep),
        // To implement
        Edge::new("e-design-impl", "design", "implement")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::Bezier)
            .with_animated(true),
        Edge::new("e-research-impl", "research", "implement")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::Bezier),
        // To test
        Edge::new("e-research-test", "research", "test")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::Step),
        // To review
        Edge::new("e-impl-review", "implement", "review")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::Bezier),
        Edge::new("e-test-review", "test", "review")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::Bezier),
        // To deploy
        Edge::new("e-review-deploy", "review", "deploy")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::Straight)
            .with_label("Approved"),
    ];

    let state: Signal<FlowState<TaskData>> = use_signal(|| {
        FlowState::with_nodes_and_edges(initial_nodes, initial_edges)
    });

    // UI state
    let mut bg_variant = use_signal(|| BackgroundVariant::Dots);
    let mut snap_enabled = use_signal(|| false);
    let snap_size = 20.0;

    let cycle_background = move |_| {
        let current = *bg_variant.read();
        bg_variant.set(match current {
            BackgroundVariant::Dots => BackgroundVariant::Lines,
            BackgroundVariant::Lines => BackgroundVariant::Cross,
            BackgroundVariant::Cross => BackgroundVariant::Dots,
        });
    };

    let toggle_snap = move |_| {
        let current = *snap_enabled.read();
        snap_enabled.set(!current);
    };

    let bg = *bg_variant.read();
    let snap = *snap_enabled.read();

    let bg_name = match bg {
        BackgroundVariant::Dots => "Dots",
        BackgroundVariant::Lines => "Lines",
        BackgroundVariant::Cross => "Cross",
    };
    let snap_text = if snap { "ON" } else { "OFF" };
    let snap_class = if snap { "active" } else { "" };
    let gap = if snap { snap_size } else { 20.0 };

    rsx! {
        style { "{FLOW_STYLES}" }
        style { "{CONTROLS_STYLES}" }
        style { "{CUSTOM_STYLES}" }

        div {
            class: "app-container",

            // Header
            div {
                class: "header",
                h1 { "dioxus-flow Feature Showcase" }
                div {
                    class: "header-controls",
                    button {
                        onclick: cycle_background,
                        "Background: {bg_name}"
                    }
                    button {
                        onclick: toggle_snap,
                        class: "{snap_class}",
                        "Snap to Grid: {snap_text}"
                    }
                }
            }

            // Main flow area
            div {
                class: "flow-area",

                // Background
                Background {
                    variant: bg,
                    gap: gap,
                    color: "#e0e0e0",
                    background_color: "#fafafa",
                }

                // Main flow
                Flow {
                    state: state,
                    on_node_click: move |id: String| {
                        web_sys::console::log_1(&format!("Clicked: {}", id).into());
                    },
                    on_connect: move |edge: Edge| {
                        web_sys::console::log_1(&format!("Connected: {} -> {}", edge.source, edge.target).into());
                    },
                }

                // Controls (zoom buttons)
                Controls {
                    state: state,
                    position: ControlsPosition::BottomLeft,
                    show_fit_view: true,
                    show_interactive: true,
                }

                // Minimap
                MiniMap {
                    state: state,
                    position: MiniMapPosition::BottomRight,
                    width: 180.0,
                    height: 120.0,
                    node_color: "#ddd",
                    node_stroke_color: "#999",
                }
            }

            // Legend
            div {
                class: "legend",
                h3 { "Legend" }
                div { class: "legend-items",
                    div { class: "legend-item",
                        span { class: "legend-dot task-done" }
                        "Done"
                    }
                    div { class: "legend-item",
                        span { class: "legend-dot task-progress" }
                        "In Progress"
                    }
                    div { class: "legend-item",
                        span { class: "legend-dot task-todo" }
                        "To Do"
                    }
                }
                h3 { "Edge Types" }
                div { class: "legend-items",
                    span { "Bezier • Straight • Step • SmoothStep" }
                }
                h3 { "Keyboard Shortcuts" }
                div { class: "legend-items shortcuts",
                    div { "Pan: Click & drag on canvas" }
                    div { "Zoom: Scroll wheel" }
                    div { "Select: Click node" }
                    div { "Connect: Drag from handle" }
                }
            }
        }
    }
}

const CUSTOM_STYLES: &str = r#"
* {
    box-sizing: border-box;
}

body, html, #main {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #f5f5f5;
}

.header {
    padding: 12px 20px;
    background: white;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}

.header h1 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #333;
}

.header-controls {
    display: flex;
    gap: 8px;
}

.header-controls button {
    padding: 6px 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background: white;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
}

.header-controls button:hover {
    background: #f5f5f5;
    border-color: #ccc;
}

.header-controls button.active {
    background: #1a73e8;
    color: white;
    border-color: #1a73e8;
}

.flow-area {
    flex: 1;
    position: relative;
    overflow: hidden;
}

.legend {
    padding: 16px 20px;
    background: white;
    border-top: 1px solid #e0e0e0;
}

.legend h3 {
    margin: 0 0 8px 0;
    font-size: 12px;
    font-weight: 600;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.legend h3:not(:first-child) {
    margin-top: 12px;
}

.legend-items {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    font-size: 13px;
    color: #555;
}

.legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
}

.legend-dot {
    width: 12px;
    height: 12px;
    border-radius: 3px;
}

.shortcuts {
    flex-direction: column;
    gap: 4px;
}

.shortcuts div {
    font-size: 12px;
    color: #777;
}

/* Task node styles */
.dioxus-flow-node {
    min-width: 140px;
    border-radius: 8px;
    border: 2px solid;
    padding: 12px 16px;
}

.dioxus-flow-node-task-todo {
    background: #fff;
    border-color: #e0e0e0;
}

.dioxus-flow-node-task-progress {
    background: #fff8e1;
    border-color: #ffc107;
}

.dioxus-flow-node-task-done {
    background: #e8f5e9;
    border-color: #4caf50;
}

.task-todo { background: #e0e0e0; }
.task-progress { background: #ffc107; }
.task-done { background: #4caf50; }

/* Handle styles */
.dioxus-flow-handle {
    width: 12px;
    height: 12px;
    background: #1a73e8;
    border: 2px solid white;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}

.dioxus-flow-handle:hover {
    transform: scale(1.2);
}

/* Edge styles */
.dioxus-flow-edge-path {
    stroke: #90a4ae;
    stroke-width: 2;
}

.dioxus-flow-edge-animated .dioxus-flow-edge-path {
    stroke: #1a73e8;
    stroke-dasharray: 5;
    animation: dash 0.5s linear infinite;
}

@keyframes dash {
    to { stroke-dashoffset: -10; }
}

/* Minimap styles */
.dioxus-flow-minimap {
    border-radius: 8px !important;
    box-shadow: 0 2px 8px rgba(0,0,0,0.15) !important;
}

/* Controls styles */
.dioxus-flow-controls {
    background: white;
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.dioxus-flow-controls-button {
    border-radius: 6px !important;
}
"#;

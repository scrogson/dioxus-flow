//! Example demonstrating multiple input/output handles per node.
//!
//! This example shows a data processing pipeline where nodes can have
//! multiple inputs and outputs, similar to visual programming tools
//! like Unreal Blueprints, Blender nodes, or data pipeline builders.

use dioxus::prelude::*;
use dioxus_flow::components::flow::FLOW_STYLES;
use dioxus_flow::components::controls::CONTROLS_STYLES;
use dioxus_flow::hooks::FlowState;
use dioxus_flow::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq, Default)]
struct ProcessorData {
    name: String,
    processor_type: ProcessorType,
}

#[derive(Clone, PartialEq, Default)]
enum ProcessorType {
    #[default]
    DataSource,
    Transform,
    Filter,
    Join,
    Aggregate,
    Output,
}

impl ProcessorData {
    fn new(name: &str, processor_type: ProcessorType) -> Self {
        Self {
            name: name.to_string(),
            processor_type,
        }
    }
}

#[component]
fn App() -> Element {
    // Create nodes with multiple handles
    let initial_nodes = vec![
        // Data Sources (outputs only)
        Node::new_without_handles("users", 50.0, 100.0)
            .with_label("Users DB")
            .with_data(ProcessorData::new("Users DB", ProcessorType::DataSource))
            .with_type("source")
            .with_dimensions(160.0, 60.0)
            .with_outputs(&["users"]),
        Node::new_without_handles("orders", 50.0, 250.0)
            .with_label("Orders DB")
            .with_data(ProcessorData::new("Orders DB", ProcessorType::DataSource))
            .with_type("source")
            .with_dimensions(160.0, 60.0)
            .with_outputs(&["orders"]),
        Node::new_without_handles("products", 50.0, 400.0)
            .with_label("Products API")
            .with_data(ProcessorData::new("Products API", ProcessorType::DataSource))
            .with_type("source")
            .with_dimensions(160.0, 60.0)
            .with_outputs(&["products"]),

        // Join node (multiple inputs, one output)
        Node::new_without_handles("join", 300.0, 175.0)
            .with_label("Join")
            .with_data(ProcessorData::new("Join", ProcessorType::Join))
            .with_type("join")
            .with_dimensions(140.0, 100.0)
            .with_inputs(&["left", "right"])
            .with_outputs(&["joined"]),

        // Transform node
        Node::new_without_handles("transform", 500.0, 100.0)
            .with_label("Transform")
            .with_data(ProcessorData::new("Transform", ProcessorType::Transform))
            .with_type("transform")
            .with_dimensions(160.0, 80.0)
            .with_inputs(&["data"])
            .with_outputs(&["transformed", "errors"]),

        // Filter node
        Node::new_without_handles("filter", 500.0, 280.0)
            .with_label("Filter")
            .with_data(ProcessorData::new("Filter", ProcessorType::Filter))
            .with_type("filter")
            .with_dimensions(140.0, 60.0)
            .with_inputs(&["input"])
            .with_outputs(&["passed", "rejected"]),

        // Aggregate node (multiple inputs, multiple outputs)
        Node::new_without_handles("aggregate", 720.0, 180.0)
            .with_label("Aggregate")
            .with_data(ProcessorData::new("Aggregate", ProcessorType::Aggregate))
            .with_type("aggregate")
            .with_dimensions(160.0, 100.0)
            .with_inputs(&["data1", "data2", "data3"])
            .with_outputs(&["sum", "avg", "count"]),

        // Output nodes (inputs only)
        Node::new_without_handles("dashboard", 950.0, 80.0)
            .with_label("Dashboard")
            .with_data(ProcessorData::new("Dashboard", ProcessorType::Output))
            .with_type("output")
            .with_dimensions(140.0, 50.0)
            .with_inputs(&["metrics"]),
        Node::new_without_handles("report", 950.0, 180.0)
            .with_label("Report")
            .with_data(ProcessorData::new("Report", ProcessorType::Output))
            .with_type("output")
            .with_dimensions(140.0, 50.0)
            .with_inputs(&["data"]),
        Node::new_without_handles("alert", 950.0, 280.0)
            .with_label("Alerts")
            .with_data(ProcessorData::new("Alerts", ProcessorType::Output))
            .with_type("output")
            .with_dimensions(140.0, 50.0)
            .with_inputs(&["errors"]),
    ];

    let initial_edges = vec![
        // Connect sources to join
        Edge::new_with_handles("e1", "users", "output-0", "join", "input-0")
            .with_type(EdgeType::SmoothStep)
            .with_label("users"),
        Edge::new_with_handles("e2", "orders", "output-0", "join", "input-1")
            .with_type(EdgeType::SmoothStep)
            .with_label("orders"),

        // Connect join to transform
        Edge::new_with_handles("e3", "join", "output-0", "transform", "input-0")
            .with_type(EdgeType::Bezier)
            .with_label("joined"),

        // Connect products to filter
        Edge::new_with_handles("e4", "products", "output-0", "filter", "input-0")
            .with_type(EdgeType::SmoothStep)
            .with_label("products"),

        // Connect to aggregate
        Edge::new_with_handles("e5", "transform", "output-0", "aggregate", "input-0")
            .with_type(EdgeType::Bezier)
            .with_label("data"),
        Edge::new_with_handles("e6", "filter", "output-0", "aggregate", "input-1")
            .with_type(EdgeType::Bezier)
            .with_label("filtered"),

        // Connect aggregate outputs to destinations
        Edge::new_with_handles("e7", "aggregate", "output-0", "dashboard", "input-0")
            .with_type(EdgeType::SmoothStep)
            .with_label("metrics"),
        Edge::new_with_handles("e8", "aggregate", "output-1", "report", "input-0")
            .with_type(EdgeType::SmoothStep)
            .with_label("report"),

        // Connect error outputs
        Edge::new_with_handles("e9", "transform", "output-1", "alert", "input-0")
            .with_type(EdgeType::Bezier)
            .with_stroke("#ef4444")
            .with_animated(true)
            .with_label("errors"),
    ];

    let state: Signal<FlowState<ProcessorData>> = use_signal(|| {
        FlowState::with_nodes_and_edges(initial_nodes, initial_edges)
    });

    rsx! {
        style { "{FLOW_STYLES}" }
        style { "{CONTROLS_STYLES}" }
        style { "{MULTI_HANDLE_STYLES}" }

        div {
            class: "app-container",

            div {
                class: "header",
                h1 { "Data Pipeline Builder" }
                p { "Nodes with multiple input/output handles" }
            }

            div {
                class: "flow-area",

                Background {
                    variant: BackgroundVariant::Dots,
                    gap: 20.0,
                    color: "#e5e5e5",
                    background_color: "#fafafa",
                }

                Flow {
                    state: state,
                }

                Controls {
                    state: state,
                    position: ControlsPosition::BottomLeft,
                }

                MiniMap {
                    state: state,
                    position: MiniMapPosition::BottomRight,
                    width: 180.0,
                    height: 120.0,
                }
            }

            div {
                class: "info-panel",
                h3 { "Node Types" }
                div { class: "node-type",
                    span { class: "dot source" }
                    "Data Source - Output only"
                }
                div { class: "node-type",
                    span { class: "dot transform" }
                    "Transform - 1 input, 2 outputs"
                }
                div { class: "node-type",
                    span { class: "dot join" }
                    "Join - 2 inputs, 1 output"
                }
                div { class: "node-type",
                    span { class: "dot aggregate" }
                    "Aggregate - 3 inputs, 3 outputs"
                }
                div { class: "node-type",
                    span { class: "dot output" }
                    "Output - Input only"
                }
            }
        }
    }
}

const MULTI_HANDLE_STYLES: &str = r#"
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
    background: #1a1a2e;
}

.header {
    padding: 16px 24px;
    background: #16213e;
    color: white;
    border-bottom: 1px solid #0f3460;
}

.header h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
}

.header p {
    margin: 4px 0 0 0;
    font-size: 13px;
    color: #94a3b8;
}

.flow-area {
    flex: 1;
    position: relative;
    overflow: hidden;
}

.info-panel {
    padding: 12px 20px;
    background: #16213e;
    border-top: 1px solid #0f3460;
}

.info-panel h3 {
    margin: 0 0 8px 0;
    font-size: 11px;
    font-weight: 600;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.node-type {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-right: 20px;
    font-size: 12px;
    color: #94a3b8;
}

.dot {
    width: 10px;
    height: 10px;
    border-radius: 3px;
}

.dot.source { background: linear-gradient(135deg, #22c55e, #16a34a); }
.dot.transform { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.dot.join { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.dot.aggregate { background: linear-gradient(135deg, #f59e0b, #d97706); }
.dot.output { background: linear-gradient(135deg, #ef4444, #dc2626); }

/* Override flow container background */
.dioxus-flow-container {
    background-color: #1a1a2e !important;
    background-image: radial-gradient(#333355 1px, transparent 1px) !important;
    background-size: 24px 24px !important;
}

/* Node styling */
.dioxus-flow-node {
    border-radius: 12px;
    border: none;
    font-size: 13px;
    font-weight: 600;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255,255,255,0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    text-shadow: 0 1px 2px rgba(0,0,0,0.2);
    cursor: grab;
    user-select: none;
}

.dioxus-flow-node:active {
    cursor: grabbing;
}

.dioxus-flow-node-content {
    padding: 12px 16px;
    text-align: center;
    pointer-events: none;
}

.dioxus-flow-node-source {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
    color: white;
}

.dioxus-flow-node-transform {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    color: white;
}

.dioxus-flow-node-join {
    background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
    color: white;
}

.dioxus-flow-node-filter {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    color: white;
}

.dioxus-flow-node-aggregate {
    background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
    color: white;
}

.dioxus-flow-node-output {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    color: white;
}

.dioxus-flow-node-selected {
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4), 0 0 0 2px #60a5fa !important;
}

/* Handle styling */
.dioxus-flow-handle {
    width: 10px;
    height: 10px;
    background: white;
    border: 2px solid #64748b;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0,0,0,0.3);
    cursor: crosshair;
    z-index: 10;
    transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.dioxus-flow-handle-source {
    border-color: #22c55e;
    background: #dcfce7;
}

.dioxus-flow-handle-target {
    border-color: #3b82f6;
    background: #dbeafe;
}

.dioxus-flow-handle:hover {
    border-color: #60a5fa;
    background: white;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.3), 0 2px 4px rgba(0,0,0,0.3);
}

/* Edge styling */
.dioxus-flow-edge-path {
    stroke: #64748b;
    stroke-width: 2;
    transition: stroke 0.2s ease;
}

.dioxus-flow-edge:hover .dioxus-flow-edge-path {
    stroke: #94a3b8;
}

.dioxus-flow-edge-selected .dioxus-flow-edge-path {
    stroke: #60a5fa;
    stroke-width: 2.5;
}

.dioxus-flow-edge-animated .dioxus-flow-edge-path {
    stroke-dasharray: 5;
    animation: dash 0.5s linear infinite;
}

/* Edge label styling */
.dioxus-flow-edge-label {
    background: #1e293b;
    color: #94a3b8;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 500;
    border: 1px solid #334155;
    white-space: nowrap;
}

@keyframes dash {
    to { stroke-dashoffset: -10; }
}

/* MiniMap override */
.dioxus-flow-minimap {
    background: #16213e !important;
    border: 1px solid #0f3460 !important;
}

/* Controls override */
.dioxus-flow-controls {
    background: #16213e !important;
    border: 1px solid #0f3460 !important;
}

.dioxus-flow-controls button {
    background: #1e293b !important;
    color: #94a3b8 !important;
    border-color: #334155 !important;
}

.dioxus-flow-controls button:hover {
    background: #334155 !important;
    color: white !important;
}
"#;

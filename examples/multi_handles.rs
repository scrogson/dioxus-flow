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
            .with_data(ProcessorData::new("Users DB", ProcessorType::DataSource))
            .with_type("source")
            .with_dimensions(160.0, 60.0)
            .with_outputs(&["users"]),
        Node::new_without_handles("orders", 50.0, 250.0)
            .with_data(ProcessorData::new("Orders DB", ProcessorType::DataSource))
            .with_type("source")
            .with_dimensions(160.0, 60.0)
            .with_outputs(&["orders"]),
        Node::new_without_handles("products", 50.0, 400.0)
            .with_data(ProcessorData::new("Products API", ProcessorType::DataSource))
            .with_type("source")
            .with_dimensions(160.0, 60.0)
            .with_outputs(&["products"]),

        // Join node (multiple inputs, one output)
        Node::new_without_handles("join", 300.0, 175.0)
            .with_data(ProcessorData::new("Join", ProcessorType::Join))
            .with_type("join")
            .with_dimensions(140.0, 100.0)
            .with_inputs(&["left", "right"])
            .with_outputs(&["joined"]),

        // Transform node
        Node::new_without_handles("transform", 500.0, 100.0)
            .with_data(ProcessorData::new("Transform", ProcessorType::Transform))
            .with_type("transform")
            .with_dimensions(160.0, 80.0)
            .with_inputs(&["data"])
            .with_outputs(&["transformed", "errors"]),

        // Filter node
        Node::new_without_handles("filter", 500.0, 280.0)
            .with_data(ProcessorData::new("Filter", ProcessorType::Filter))
            .with_type("filter")
            .with_dimensions(140.0, 60.0)
            .with_inputs(&["input"])
            .with_outputs(&["passed", "rejected"]),

        // Aggregate node (multiple inputs, multiple outputs)
        Node::new_without_handles("aggregate", 720.0, 180.0)
            .with_data(ProcessorData::new("Aggregate", ProcessorType::Aggregate))
            .with_type("aggregate")
            .with_dimensions(160.0, 100.0)
            .with_inputs(&["data1", "data2", "data3"])
            .with_outputs(&["sum", "avg", "count"]),

        // Output nodes (inputs only)
        Node::new_without_handles("dashboard", 950.0, 80.0)
            .with_data(ProcessorData::new("Dashboard", ProcessorType::Output))
            .with_type("output")
            .with_dimensions(140.0, 50.0)
            .with_inputs(&["metrics"]),
        Node::new_without_handles("report", 950.0, 180.0)
            .with_data(ProcessorData::new("Report", ProcessorType::Output))
            .with_type("output")
            .with_dimensions(140.0, 50.0)
            .with_inputs(&["data"]),
        Node::new_without_handles("alert", 950.0, 280.0)
            .with_data(ProcessorData::new("Alerts", ProcessorType::Output))
            .with_type("output")
            .with_dimensions(140.0, 50.0)
            .with_inputs(&["errors"]),
    ];

    let initial_edges = vec![
        // Connect sources to join
        Edge::new_with_handles("e1", "users", "output-0", "join", "input-0")
            .with_type(EdgeType::SmoothStep),
        Edge::new_with_handles("e2", "orders", "output-0", "join", "input-1")
            .with_type(EdgeType::SmoothStep),

        // Connect join to transform
        Edge::new_with_handles("e3", "join", "output-0", "transform", "input-0")
            .with_type(EdgeType::Bezier),

        // Connect products to filter
        Edge::new_with_handles("e4", "products", "output-0", "filter", "input-0")
            .with_type(EdgeType::SmoothStep),

        // Connect to aggregate
        Edge::new_with_handles("e5", "transform", "output-0", "aggregate", "input-0")
            .with_type(EdgeType::Bezier),
        Edge::new_with_handles("e6", "filter", "output-0", "aggregate", "input-1")
            .with_type(EdgeType::Bezier),

        // Connect aggregate outputs to destinations
        Edge::new_with_handles("e7", "aggregate", "output-0", "dashboard", "input-0")
            .with_type(EdgeType::SmoothStep),
        Edge::new_with_handles("e8", "aggregate", "output-1", "report", "input-0")
            .with_type(EdgeType::SmoothStep),

        // Connect error outputs
        Edge::new_with_handles("e9", "transform", "output-1", "alert", "input-0")
            .with_type(EdgeType::Bezier)
            .with_stroke("#ef4444")
            .with_animated(true),
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
    background: #f0f0f0;
}

.header {
    padding: 12px 20px;
    background: #1e293b;
    color: white;
}

.header h1 {
    margin: 0;
    font-size: 18px;
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
    background: white;
    border-top: 1px solid #e5e5e5;
}

.info-panel h3 {
    margin: 0 0 8px 0;
    font-size: 12px;
    font-weight: 600;
    color: #64748b;
    text-transform: uppercase;
}

.node-type {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-right: 20px;
    font-size: 12px;
    color: #475569;
}

.dot {
    width: 10px;
    height: 10px;
    border-radius: 2px;
}

.dot.source { background: #22c55e; }
.dot.transform { background: #3b82f6; }
.dot.join { background: #8b5cf6; }
.dot.aggregate { background: #f59e0b; }
.dot.output { background: #ef4444; }

/* Node styling */
.dioxus-flow-node {
    border-radius: 8px;
    border: 2px solid;
    font-size: 13px;
    font-weight: 500;
}

.dioxus-flow-node-source {
    background: linear-gradient(135deg, #dcfce7, #bbf7d0);
    border-color: #22c55e;
    color: #166534;
}

.dioxus-flow-node-transform {
    background: linear-gradient(135deg, #dbeafe, #bfdbfe);
    border-color: #3b82f6;
    color: #1e40af;
}

.dioxus-flow-node-join {
    background: linear-gradient(135deg, #ede9fe, #ddd6fe);
    border-color: #8b5cf6;
    color: #5b21b6;
}

.dioxus-flow-node-filter {
    background: linear-gradient(135deg, #fef3c7, #fde68a);
    border-color: #f59e0b;
    color: #92400e;
}

.dioxus-flow-node-aggregate {
    background: linear-gradient(135deg, #ffedd5, #fed7aa);
    border-color: #f97316;
    color: #9a3412;
}

.dioxus-flow-node-output {
    background: linear-gradient(135deg, #fee2e2, #fecaca);
    border-color: #ef4444;
    color: #991b1b;
}

/* Handle styling */
.dioxus-flow-handle {
    width: 12px;
    height: 12px;
    background: #64748b;
    border: 2px solid white;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}

.dioxus-flow-handle-source {
    background: #22c55e;
}

.dioxus-flow-handle-target {
    background: #3b82f6;
}

.dioxus-flow-handle:hover {
    transform: scale(1.2);
}

/* Edge styling */
.dioxus-flow-edge-path {
    stroke: #94a3b8;
    stroke-width: 2;
}

.dioxus-flow-edge-animated .dioxus-flow-edge-path {
    stroke-dasharray: 5;
    animation: dash 0.5s linear infinite;
}

@keyframes dash {
    to { stroke-dashoffset: -10; }
}
"#;

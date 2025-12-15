//! Example with custom node types and styling.

use dioxus::prelude::*;
use dioxus_flow::components::flow::FLOW_STYLES;
use dioxus_flow::hooks::FlowState;
use dioxus_flow::prelude::*;

fn main() {
    dioxus::launch(App);
}

/// Custom data for nodes
#[derive(Clone, PartialEq, Default)]
struct NodeData {
    label: String,
    description: String,
    #[allow(dead_code)]
    color: String,
}

impl NodeData {
    fn new(label: &str, description: &str, color: &str) -> Self {
        Self {
            label: label.to_string(),
            description: description.to_string(),
            color: color.to_string(),
        }
    }
}

#[component]
fn App() -> Element {
    // Create nodes with custom data
    let initial_nodes = vec![
        Node::new("start", 50.0, 200.0)
            .with_data(NodeData::new("Start", "Begin workflow", "#22c55e"))
            .with_type("start"),
        Node::new("process1", 250.0, 100.0)
            .with_data(NodeData::new("Process A", "First processing step", "#3b82f6"))
            .with_type("process"),
        Node::new("process2", 250.0, 300.0)
            .with_data(NodeData::new("Process B", "Alternative path", "#3b82f6"))
            .with_type("process"),
        Node::new("decision", 450.0, 200.0)
            .with_data(NodeData::new("Decision", "Choose path", "#f59e0b"))
            .with_type("decision"),
        Node::new("end", 650.0, 200.0)
            .with_data(NodeData::new("End", "Workflow complete", "#ef4444"))
            .with_type("end"),
    ];

    let initial_edges = vec![
        Edge::new("e1", "start", "process1")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_label("Path A"),
        Edge::new("e2", "start", "process2")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_label("Path B"),
        Edge::new("e3", "process1", "decision")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left),
        Edge::new("e4", "process2", "decision")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left),
        Edge::new("e5", "decision", "end")
            .with_source_handle(HandlePosition::Right)
            .with_target_handle(HandlePosition::Left)
            .with_type(EdgeType::SmoothStep)
            .with_animated(true),
    ];

    let mut state: Signal<FlowState<NodeData>> = use_signal(|| {
        FlowState::with_nodes_and_edges(initial_nodes, initial_edges)
    });

    // Controls for the flow
    let mut node_count = use_signal(|| 5);

    let add_node = move |_| {
        let count = *node_count.read() + 1;
        node_count.set(count);

        let new_node = Node::new(
            format!("node{}", count),
            100.0 + (count as f64 * 50.0) % 400.0,
            100.0 + (count as f64 * 30.0) % 300.0,
        )
        .with_data(NodeData::new(
            &format!("Node {}", count),
            "Dynamically added",
            "#8b5cf6",
        ))
        .with_type("dynamic");

        state.write().add_node(new_node);
    };

    let fit_view = move |_| {
        // Reset viewport to default
        state.write().set_viewport(Viewport::default());
    };

    rsx! {
        style { "{FLOW_STYLES}" }
        style { r#"
            body, html, #main {{
                margin: 0;
                padding: 0;
                width: 100%;
                height: 100vh;
                font-family: system-ui, -apple-system, sans-serif;
            }}
            .app-container {{
                display: flex;
                flex-direction: column;
                width: 100%;
                height: 100vh;
            }}
            .toolbar {{
                padding: 10px;
                background: #1a1a2e;
                color: white;
                display: flex;
                gap: 10px;
                align-items: center;
            }}
            .toolbar button {{
                padding: 8px 16px;
                border: none;
                border-radius: 4px;
                background: #4a4a6a;
                color: white;
                cursor: pointer;
                font-size: 14px;
            }}
            .toolbar button:hover {{
                background: #5a5a7a;
            }}
            .flow-wrapper {{
                flex: 1;
                width: 100%;
            }}
            .dioxus-flow-node {{
                min-width: 120px;
            }}
            .dioxus-flow-node-start {{
                background: linear-gradient(135deg, #22c55e, #16a34a);
                color: white;
                border: none;
            }}
            .dioxus-flow-node-process {{
                background: linear-gradient(135deg, #3b82f6, #2563eb);
                color: white;
                border: none;
            }}
            .dioxus-flow-node-decision {{
                background: linear-gradient(135deg, #f59e0b, #d97706);
                color: white;
                border: none;
            }}
            .dioxus-flow-node-end {{
                background: linear-gradient(135deg, #ef4444, #dc2626);
                color: white;
                border: none;
            }}
            .dioxus-flow-node-dynamic {{
                background: linear-gradient(135deg, #8b5cf6, #7c3aed);
                color: white;
                border: none;
            }}
        "# }

        div {
            class: "app-container",

            div {
                class: "toolbar",
                span { "Custom Nodes Example" }
                button {
                    onclick: add_node,
                    "Add Node"
                }
                button {
                    onclick: fit_view,
                    "Reset View"
                }
            }

            div {
                class: "flow-wrapper",
                Flow {
                    state: state,
                    on_connect: move |_edge: Edge| {
                        // Handle new connections
                    },
                }
            }
        }
    }
}

//! Basic example demonstrating dioxus-flow usage.

use dioxus::prelude::*;
use dioxus_flow::components::flow::FLOW_STYLES;
use dioxus_flow::hooks::FlowState;
use dioxus_flow::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Create initial nodes
    let initial_nodes = vec![
        Node::new("1", 100.0, 100.0)
            .with_data("Input Node".to_string())
            .with_type("input"),
        Node::new("2", 300.0, 200.0)
            .with_data("Process Node".to_string())
            .with_type("default"),
        Node::new("3", 500.0, 100.0)
            .with_data("Output Node".to_string())
            .with_type("output"),
    ];

    // Create initial edges
    let initial_edges = vec![
        Edge::new("e1-2", "1", "2")
            .with_animated(true),
        Edge::new("e2-3", "2", "3"),
    ];

    // Initialize flow state
    let state: Signal<FlowState<String>> = use_signal(|| {
        FlowState::with_nodes_and_edges(initial_nodes, initial_edges)
    });

    rsx! {
        style { "{FLOW_STYLES}" }
        style { r#"
            body, html, #main {{
                margin: 0;
                padding: 0;
                width: 100%;
                height: 100vh;
            }}
            .flow-wrapper {{
                width: 100%;
                height: 100vh;
            }}
            .dioxus-flow-node-input {{
                background: #d4edda;
                border-color: #28a745;
            }}
            .dioxus-flow-node-output {{
                background: #cce5ff;
                border-color: #007bff;
            }}
        "# }

        div {
            class: "flow-wrapper",
            Flow {
                state: state,
                on_node_click: move |_node_id: String| {
                    // Node clicked - handle as needed
                },
                on_connect: move |_edge: Edge| {
                    // New connection made - handle as needed
                },
                on_pane_click: move |_pos: Position| {
                    // Pane clicked - handle as needed
                },
            }
        }
    }
}

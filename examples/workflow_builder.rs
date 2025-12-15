//! Workflow Builder Example
//!
//! A workflow automation builder similar to n8n or Node-RED,
//! featuring a node palette sidebar, custom styled nodes, and edge labels.

use dioxus::prelude::*;
use dioxus_flow::components::controls::CONTROLS_STYLES;
use dioxus_flow::components::flow::FLOW_STYLES;
use dioxus_flow::hooks::FlowState;
use dioxus_flow::prelude::*;
use dioxus_flow::types::Node;

fn main() {
    dioxus::launch(App);
}

/// Node types available in the workflow builder
#[derive(Clone, Copy, PartialEq, Default)]
enum WorkflowNodeType {
    #[default]
    Execution,
    Decision,
    Join,
    Split,
    ForEach,
    Loop,
    TryCatch,
    Http,
}

impl WorkflowNodeType {
    fn label(&self) -> &'static str {
        match self {
            Self::Execution => "Execution",
            Self::Decision => "Decision",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::ForEach => "ForEach",
            Self::Loop => "While",
            Self::TryCatch => "Try/Catch",
            Self::Http => "HTTP",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Execution => "Run code (Rhai, Lua, JS)",
            Self::Decision => "If/else branching",
            Self::Join => "Merge multiple branches",
            Self::Split => "Fan-out to parallel",
            Self::ForEach => "Iterate over collection",
            Self::Loop => "Loop with condition",
            Self::TryCatch => "Error handling",
            Self::Http => "Make HTTP requests",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Execution => "⚙",
            Self::Decision => "?",
            Self::Join => "⇄",
            Self::Split => "⚡",
            Self::ForEach => "↻",
            Self::Loop => "∞",
            Self::TryCatch => "△",
            Self::Http => "⊛",
        }
    }

    fn css_class(&self) -> &'static str {
        match self {
            Self::Execution => "node-execution",
            Self::Decision => "node-decision",
            Self::Join => "node-join",
            Self::Split => "node-split",
            Self::ForEach => "node-foreach",
            Self::Loop => "node-loop",
            Self::TryCatch => "node-trycatch",
            Self::Http => "node-http",
        }
    }
}

/// Custom data for workflow nodes
#[derive(Clone, PartialEq, Default)]
struct WorkflowNodeData {
    node_type: WorkflowNodeType,
    title: String,
    subtitle: String,
    code_preview: String,
}

impl WorkflowNodeData {
    fn new(
        node_type: WorkflowNodeType,
        title: &str,
        subtitle: &str,
        code_preview: &str,
    ) -> Self {
        Self {
            node_type,
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            code_preview: code_preview.to_string(),
        }
    }
}

#[component]
fn App() -> Element {
    // Create initial workflow nodes
    let initial_nodes = vec![
        Node::new_without_handles("fetch", 400.0, 80.0)
            .with_label("Fetch Data")
            .with_data(WorkflowNodeData::new(
                WorkflowNodeType::Execution,
                "Fetch Data",
                "rhai",
                "// Fetch data from the API...",
            ))
            .with_type("node-execution")
            .with_dimensions(200.0, 100.0)
            .with_handle(NodeHandle::target("in").with_position(HandlePosition::Top))
            .with_handle(NodeHandle::source("out").with_position(HandlePosition::Bottom)),
        Node::new_without_handles("check", 400.0, 280.0)
            .with_label("Check Response")
            .with_data(WorkflowNodeData::new(
                WorkflowNodeType::Decision,
                "Check Response",
                "condition",
                "fetch.ok == true",
            ))
            .with_type("node-decision")
            .with_dimensions(200.0, 80.0)
            .with_handle(NodeHandle::target("in").with_position(HandlePosition::Top))
            .with_handle(
                NodeHandle::source("success")
                    .with_position(HandlePosition::Left)
                    .with_label("success"),
            )
            .with_handle(
                NodeHandle::source("error")
                    .with_position(HandlePosition::Right)
                    .with_label("error"),
            ),
        Node::new_without_handles("process", 200.0, 480.0)
            .with_label("Process Success")
            .with_data(WorkflowNodeData::new(
                WorkflowNodeType::Execution,
                "Process Success",
                "rhai",
                "// Process successful response...",
            ))
            .with_type("node-execution")
            .with_dimensions(200.0, 100.0)
            .with_handle(NodeHandle::target("in").with_position(HandlePosition::Top))
            .with_handle(NodeHandle::source("out").with_position(HandlePosition::Bottom)),
        Node::new_without_handles("error", 600.0, 480.0)
            .with_label("Handle Error")
            .with_data(WorkflowNodeData::new(
                WorkflowNodeType::Execution,
                "Handle Error",
                "rhai",
                "// Handle error response #{ ...",
            ))
            .with_type("node-execution")
            .with_dimensions(200.0, 100.0)
            .with_handle(NodeHandle::target("in").with_position(HandlePosition::Top))
            .with_handle(NodeHandle::source("out").with_position(HandlePosition::Bottom)),
        Node::new_without_handles("finalize", 400.0, 680.0)
            .with_label("Finalize")
            .with_data(WorkflowNodeData::new(
                WorkflowNodeType::Join,
                "Finalize",
                "wait_any",
                "",
            ))
            .with_type("node-join")
            .with_dimensions(180.0, 70.0)
            .with_handle(
                NodeHandle::target("in1")
                    .with_position(HandlePosition::Top)
                    .with_offset(0.3),
            )
            .with_handle(
                NodeHandle::target("in2")
                    .with_position(HandlePosition::Top)
                    .with_offset(0.7),
            )
            .with_handle(NodeHandle::source("out").with_position(HandlePosition::Bottom)),
    ];

    let initial_edges = vec![
        Edge::new("e1", "fetch", "check")
            .with_source_handle(HandlePosition::Bottom)
            .with_target_handle(HandlePosition::Top)
            .with_type(EdgeType::Bezier),
        Edge::new_with_handles("e2", "check", "success", "process", "in")
            .with_type(EdgeType::Bezier)
            .with_label("success"),
        Edge::new_with_handles("e3", "check", "error", "error", "in")
            .with_type(EdgeType::Bezier)
            .with_label("error"),
        Edge::new_with_handles("e4", "process", "out", "finalize", "in1")
            .with_type(EdgeType::Bezier),
        Edge::new_with_handles("e5", "error", "out", "finalize", "in2")
            .with_type(EdgeType::Bezier),
    ];

    let mut state: Signal<FlowState<WorkflowNodeData>> =
        use_signal(|| FlowState::with_nodes_and_edges(initial_nodes, initial_edges));

    let mut node_counter = use_signal(|| 5usize);

    // Add node helper function
    let add_node = move |node_type: WorkflowNodeType,
                         state: &mut Signal<FlowState<WorkflowNodeData>>,
                         node_counter: &mut Signal<usize>| {
        let count = *node_counter.read() + 1;
        node_counter.set(count);

        let id = format!("node_{}", count);
        let new_node = Node::new_without_handles(&id, 100.0 + (count as f64 * 20.0) % 300.0, 100.0)
            .with_label(node_type.label())
            .with_data(WorkflowNodeData::new(
                node_type,
                node_type.label(),
                match node_type {
                    WorkflowNodeType::Decision => "condition",
                    WorkflowNodeType::Join => "wait_all",
                    WorkflowNodeType::Http => "GET",
                    _ => "rhai",
                },
                "",
            ))
            .with_type(node_type.css_class())
            .with_dimensions(180.0, 80.0)
            .with_handle(NodeHandle::target("in").with_position(HandlePosition::Top))
            .with_handle(NodeHandle::source("out").with_position(HandlePosition::Bottom));

        state.write().add_node(new_node);
    };

    rsx! {
        style { "{FLOW_STYLES}" }
        style { "{CONTROLS_STYLES}" }
        style { "{WORKFLOW_BUILDER_STYLES}" }

        div {
            class: "app-container",

            // Left Sidebar - Node Palette
            aside {
                class: "sidebar",

                div {
                    class: "sidebar-header",
                    h2 { "NODES" }
                }

                div {
                    class: "node-palette",

                    NodePaletteItem {
                        node_type: WorkflowNodeType::Execution,
                        on_click: move |_| add_node(WorkflowNodeType::Execution, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::Decision,
                        on_click: move |_| add_node(WorkflowNodeType::Decision, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::Join,
                        on_click: move |_| add_node(WorkflowNodeType::Join, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::Split,
                        on_click: move |_| add_node(WorkflowNodeType::Split, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::ForEach,
                        on_click: move |_| add_node(WorkflowNodeType::ForEach, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::Loop,
                        on_click: move |_| add_node(WorkflowNodeType::Loop, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::TryCatch,
                        on_click: move |_| add_node(WorkflowNodeType::TryCatch, &mut state, &mut node_counter),
                    }
                    NodePaletteItem {
                        node_type: WorkflowNodeType::Http,
                        on_click: move |_| add_node(WorkflowNodeType::Http, &mut state, &mut node_counter),
                    }
                }

                div {
                    class: "sidebar-section",
                    h3 { "WORKFLOWS" }
                    button {
                        class: "new-workflow-btn",
                        "+ New Workflow"
                    }
                }
            }

            // Main Canvas Area
            main {
                class: "canvas-area",

                div {
                    class: "canvas-header",
                    input {
                        class: "workflow-title",
                        r#type: "text",
                        value: "Untitled Workflow",
                    }
                }

                div {
                    class: "flow-container",

                    Background {
                        variant: BackgroundVariant::Dots,
                        gap: 24.0,
                        color: "#333355",
                        size: 1.5,
                        background_color: "#1a1a2e",
                    }

                    Flow {
                        state: state,
                        node_render: Callback::new(move |node: Node<WorkflowNodeData>| {
                            let data = &node.data;
                            let icon = data.node_type.icon();
                            let title = &data.title;
                            let subtitle = &data.subtitle;
                            let code_preview = &data.code_preview;
                            let node_type = data.node_type;

                            rsx! {
                                div {
                                    class: "node-header",
                                    span { class: "node-icon", "{icon}" }
                                    span { class: "node-title", "{title}" }
                                }
                                // Different rendering based on node type
                                match node_type {
                                    // Decision nodes: show condition in a code box
                                    WorkflowNodeType::Decision => rsx! {
                                        if !code_preview.is_empty() {
                                            div { class: "node-code-box", "{code_preview}" }
                                        }
                                    },
                                    // Execution nodes: show language badge + code preview
                                    WorkflowNodeType::Execution | WorkflowNodeType::TryCatch | WorkflowNodeType::Http => rsx! {
                                        if !subtitle.is_empty() {
                                            span { class: "node-badge", "{subtitle}" }
                                        }
                                        if !code_preview.is_empty() {
                                            div { class: "node-code", "{code_preview}" }
                                        }
                                    },
                                    // Join/Split/Loop nodes: show mode badge
                                    _ => rsx! {
                                        if !subtitle.is_empty() {
                                            span { class: "node-badge", "{subtitle}" }
                                        }
                                    },
                                }
                            }
                        }),
                    }

                    Controls {
                        state: state,
                        position: ControlsPosition::BottomLeft,
                    }
                }
            }

            // Right Sidebar - Properties
            aside {
                class: "properties-panel",

                div {
                    class: "panel-actions",
                    button { class: "btn", "Save" }
                    button { class: "btn", "Export" }
                    button { class: "btn btn-primary", "Run" }
                }

                div {
                    class: "panel-section",
                    h3 { "▸ WORKFLOW" }
                }

                div {
                    class: "panel-section",
                    h3 { "▸ INPUTS" }
                }

                div {
                    class: "panel-section",
                    h3 { "▸ OUTPUTS" }
                }

                div {
                    class: "panel-section",
                    h3 { "▾ RESULTS" }
                    p {
                        class: "placeholder-text",
                        "No execution yet. Click Run to execute the workflow."
                    }
                }
            }
        }
    }
}

#[component]
fn NodePaletteItem(node_type: WorkflowNodeType, on_click: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "palette-item",
            onclick: move |evt| on_click.call(evt),

            span { class: "palette-icon", "{node_type.icon()}" }
            div {
                class: "palette-info",
                span { class: "palette-label", "{node_type.label()}" }
                span { class: "palette-desc", "{node_type.description()}" }
            }
        }
    }
}

const WORKFLOW_BUILDER_STYLES: &str = r#"
* {
    box-sizing: border-box;
}

body, html, #main {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0d0d1a;
    color: #e0e0e0;
}

.app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
}

/* Left Sidebar */
.sidebar {
    width: 240px;
    background: #141428;
    border-right: 1px solid #2a2a4a;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
}

.sidebar-header {
    padding: 16px;
    border-bottom: 1px solid #2a2a4a;
}

.sidebar-header h2 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    color: #888;
    letter-spacing: 1px;
}

.node-palette {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.palette-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    color: #e0e0e0;
    transition: all 0.2s;
}

.palette-item:hover {
    background: #1e1e3a;
    border-color: #3a3a5a;
}

.palette-icon {
    font-size: 18px;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #2a2a4a;
    border-radius: 6px;
}

.palette-info {
    display: flex;
    flex-direction: column;
}

.palette-label {
    font-size: 13px;
    font-weight: 500;
}

.palette-desc {
    font-size: 11px;
    color: #888;
}

.sidebar-section {
    padding: 16px;
    border-top: 1px solid #2a2a4a;
    margin-top: auto;
}

.sidebar-section h3 {
    margin: 0 0 12px 0;
    font-size: 11px;
    font-weight: 600;
    color: #888;
    letter-spacing: 1px;
}

.new-workflow-btn {
    width: 100%;
    padding: 10px;
    background: transparent;
    border: 1px dashed #3a3a5a;
    border-radius: 8px;
    color: #888;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
}

.new-workflow-btn:hover {
    border-color: #6366f1;
    color: #6366f1;
}

/* Main Canvas */
.canvas-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.canvas-header {
    padding: 12px 16px;
    background: #141428;
    border-bottom: 1px solid #2a2a4a;
}

.workflow-title {
    background: #1e1e3a;
    border: 1px solid #3a3a5a;
    border-radius: 6px;
    padding: 8px 12px;
    color: #e0e0e0;
    font-size: 14px;
    width: 200px;
}

.flow-container {
    flex: 1;
    position: relative;
    overflow: hidden;
}

/* Right Properties Panel */
.properties-panel {
    width: 280px;
    background: #141428;
    border-left: 1px solid #2a2a4a;
    display: flex;
    flex-direction: column;
}

.panel-actions {
    padding: 12px;
    display: flex;
    gap: 8px;
    border-bottom: 1px solid #2a2a4a;
}

.btn {
    flex: 1;
    padding: 8px 12px;
    background: #2a2a4a;
    border: 1px solid #3a3a5a;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
}

.btn:hover {
    background: #3a3a5a;
}

.btn-primary {
    background: #7c3aed;
    border-color: #7c3aed;
}

.btn-primary:hover {
    background: #6d28d9;
}

.panel-section {
    padding: 12px 16px;
    border-bottom: 1px solid #2a2a4a;
}

.panel-section h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    color: #888;
}

.placeholder-text {
    margin: 12px 0 0 0;
    font-size: 12px;
    color: #666;
}

/* Flow Container Override */
.dioxus-flow-container {
    background: transparent !important;
}

/* Node Styles */
.dioxus-flow-node {
    border-radius: 12px;
    border: none;
    padding: 0;
    min-width: auto;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
    overflow: hidden;
}

.dioxus-flow-node-node-execution {
    background: linear-gradient(135deg, #7c3aed 0%, #a855f7 100%);
}

.dioxus-flow-node-node-decision {
    background: linear-gradient(135deg, #ec4899 0%, #f472b6 100%);
}

.dioxus-flow-node-node-join {
    background: linear-gradient(135deg, #06b6d4 0%, #22d3ee 100%);
}

.dioxus-flow-node-node-split {
    background: linear-gradient(135deg, #f59e0b 0%, #fbbf24 100%);
}

.dioxus-flow-node-node-foreach {
    background: linear-gradient(135deg, #10b981 0%, #34d399 100%);
}

.dioxus-flow-node-node-loop {
    background: linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%);
}

.dioxus-flow-node-node-trycatch {
    background: linear-gradient(135deg, #8b5cf6 0%, #a78bfa 100%);
}

.dioxus-flow-node-node-http {
    background: linear-gradient(135deg, #14b8a6 0%, #2dd4bf 100%);
}

.dioxus-flow-node-content {
    padding: 12px 16px;
    color: white;
    font-weight: 500;
    font-size: 14px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
}

/* Custom node content styles */
.node-header {
    display: flex;
    align-items: center;
    gap: 8px;
}

.node-icon {
    font-size: 16px;
    opacity: 0.9;
}

.node-title {
    font-size: 15px;
    font-weight: 600;
}

.node-badge {
    display: inline-block;
    background: rgba(0, 0, 0, 0.25);
    padding: 4px 10px;
    border-radius: 4px;
    font-family: 'SF Mono', Monaco, 'Courier New', monospace;
    font-size: 12px;
    font-weight: 500;
}

.node-code {
    font-family: 'SF Mono', Monaco, 'Courier New', monospace;
    font-size: 11px;
    opacity: 0.85;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
}

.node-code-box {
    background: rgba(0, 0, 0, 0.2);
    padding: 8px 12px;
    border-radius: 6px;
    font-family: 'SF Mono', Monaco, 'Courier New', monospace;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 4px;
}

.dioxus-flow-node-selected {
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4), 0 0 0 2px #6366f1 !important;
}

/* Handle Styles */
.dioxus-flow-handle {
    width: 12px;
    height: 12px;
    background: white;
    border: 2px solid #666;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.dioxus-flow-handle:hover {
    border-color: #6366f1;
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.3);
}

/* Edge Styles */
.dioxus-flow-edge-path {
    stroke: #6b7280;
    stroke-width: 2;
}

.dioxus-flow-edge-selected .dioxus-flow-edge-path {
    stroke: #6366f1;
}

.dioxus-flow-edge-label {
    background: #1e1e3a;
    color: #e0e0e0;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    border: 1px solid #3a3a5a;
}

/* Controls Override */
.dioxus-flow-controls {
    background: #1e1e3a !important;
    border: 1px solid #3a3a5a !important;
    border-radius: 8px !important;
}

.dioxus-flow-controls button {
    background: transparent !important;
    color: #888 !important;
    border-color: #3a3a5a !important;
}

.dioxus-flow-controls button:hover {
    background: #2a2a4a !important;
    color: #e0e0e0 !important;
}
"#;

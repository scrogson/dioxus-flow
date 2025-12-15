//! Example: Rendering GitHub Actions-like workflows as flow diagrams.

use dioxus::prelude::*;
use dioxus_flow::components::flow::FLOW_STYLES;
use dioxus_flow::hooks::FlowState;
use dioxus_flow::prelude::*;
use std::collections::HashMap;

fn main() {
    dioxus::launch(App);
}

/// Represents a workflow job
#[derive(Clone, PartialEq, Default)]
struct JobData {
    name: String,
    runs_on: String,
    steps: Vec<String>,
    status: JobStatus,
}

#[derive(Clone, PartialEq, Default)]
enum JobStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

impl JobStatus {
    fn class(&self) -> &'static str {
        match self {
            JobStatus::Pending => "job-pending",
            JobStatus::Running => "job-running",
            JobStatus::Success => "job-success",
            JobStatus::Failed => "job-failed",
            JobStatus::Skipped => "job-skipped",
        }
    }
}

/// Parse a workflow YAML-like structure into nodes and edges
fn parse_workflow(yaml: &str) -> (Vec<Node<JobData>>, Vec<Edge>) {
    // Simple parser for demo - in production use serde_yaml
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut job_positions: HashMap<String, (f64, f64)> = HashMap::new();

    // Parse jobs section
    let mut current_job: Option<String> = None;
    let mut current_data = JobData::default();
    let mut needs: Vec<String> = Vec::new();
    let mut job_count = 0;

    for line in yaml.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("name:") {
            // Workflow name - skip for now
        } else if trimmed == "jobs:" {
            // Jobs section starts
        } else if !trimmed.starts_with('-') && !trimmed.starts_with("needs:")
            && !trimmed.starts_with("runs-on:") && !trimmed.starts_with("steps:")
            && !trimmed.starts_with("name:") && !trimmed.starts_with("run:")
            && !trimmed.starts_with("uses:") && !trimmed.is_empty()
            && trimmed.ends_with(':')
        {
            // New job definition
            if let Some(job_id) = current_job.take() {
                // Save previous job
                let (x, y) = calculate_position(job_count, &needs, &job_positions);
                job_positions.insert(job_id.clone(), (x, y));

                let node = Node::new(&job_id, x, y)
                    .with_data(current_data.clone())
                    .with_type(current_data.status.class());
                nodes.push(node);

                // Create edges from dependencies
                for dep in &needs {
                    edges.push(
                        Edge::new(format!("e-{}-{}", dep, job_id), dep.clone(), job_id.clone())
                            .with_source_handle(HandlePosition::Bottom)
                            .with_target_handle(HandlePosition::Top)
                            .with_animated(current_data.status == JobStatus::Running),
                    );
                }

                job_count += 1;
            }

            let job_id = trimmed.trim_end_matches(':').to_string();
            current_job = Some(job_id.clone());
            current_data = JobData {
                name: job_id,
                ..Default::default()
            };
            needs.clear();
        } else if trimmed.starts_with("name:") && current_job.is_some() {
            current_data.name = trimmed.trim_start_matches("name:").trim().trim_matches('"').to_string();
        } else if trimmed.starts_with("runs-on:") {
            current_data.runs_on = trimmed.trim_start_matches("runs-on:").trim().to_string();
        } else if trimmed.starts_with("needs:") {
            let deps = trimmed.trim_start_matches("needs:").trim();
            if deps.starts_with('[') {
                // Array format: [job1, job2]
                let deps = deps.trim_matches(|c| c == '[' || c == ']');
                needs = deps.split(',').map(|s| s.trim().to_string()).collect();
            } else {
                // Single dependency
                needs.push(deps.to_string());
            }
        } else if trimmed.starts_with("- run:") || trimmed.starts_with("- uses:") {
            let step = trimmed.trim_start_matches("- run:").trim_start_matches("- uses:").trim();
            current_data.steps.push(step.to_string());
        }
    }

    // Don't forget the last job
    if let Some(job_id) = current_job {
        let (x, y) = calculate_position(job_count, &needs, &job_positions);
        job_positions.insert(job_id.clone(), (x, y));

        let node = Node::new(&job_id, x, y)
            .with_data(current_data.clone())
            .with_type(current_data.status.class());
        nodes.push(node);

        for dep in &needs {
            edges.push(
                Edge::new(format!("e-{}-{}", dep, job_id), dep.clone(), job_id.clone())
                    .with_source_handle(HandlePosition::Bottom)
                    .with_target_handle(HandlePosition::Top),
            );
        }
    }

    (nodes, edges)
}

/// Calculate node position based on dependencies
fn calculate_position(
    job_index: usize,
    needs: &[String],
    positions: &HashMap<String, (f64, f64)>,
) -> (f64, f64) {
    if needs.is_empty() {
        // No dependencies - place at the top
        (150.0 + (job_index as f64 * 250.0), 50.0)
    } else {
        // Place below dependencies
        let mut max_y = 0.0f64;
        let mut avg_x = 0.0f64;

        for dep in needs {
            if let Some((x, y)) = positions.get(dep) {
                max_y = max_y.max(*y);
                avg_x += x;
            }
        }

        avg_x /= needs.len() as f64;
        (avg_x, max_y + 150.0)
    }
}

// Sample workflow YAML
const SAMPLE_WORKFLOW: &str = r#"
name: CI/CD Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    name: Lint Code
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm run lint

  test:
    name: Run Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm test

  build:
    name: Build Application
    runs-on: ubuntu-latest
    needs: [lint, test]
    steps:
      - uses: actions/checkout@v4
      - run: npm run build

  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    needs: build
    steps:
      - run: deploy --env staging

  deploy-prod:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: deploy-staging
    steps:
      - run: deploy --env production
"#;

#[component]
fn App() -> Element {
    let (initial_nodes, initial_edges) = parse_workflow(SAMPLE_WORKFLOW);

    let mut state: Signal<FlowState<JobData>> = use_signal(|| {
        FlowState::with_nodes_and_edges(initial_nodes, initial_edges)
    });

    // Simulate running workflow
    let mut current_step = use_signal(|| 0usize);

    let run_workflow = move |_| {
        current_step.set(0);
        // Reset all to pending
        let mut s = state.write();
        for node in &mut s.nodes {
            node.data.status = JobStatus::Pending;
            node.node_type = "job-pending".to_string();
        }
    };

    let step_forward = move |_| {
        let step = *current_step.read();
        let job_order = ["lint", "test", "build", "deploy-staging", "deploy-prod"];

        if step < job_order.len() {
            let mut s = state.write();

            // Mark previous as success
            if step > 0 {
                if let Some(prev_node) = s.nodes.iter_mut().find(|n| n.id == job_order[step - 1]) {
                    prev_node.data.status = JobStatus::Success;
                    prev_node.node_type = "job-success".to_string();
                }
                // For parallel jobs (lint and test both at step 0 effectively)
                if step == 2 {
                    if let Some(node) = s.nodes.iter_mut().find(|n| n.id == "test") {
                        node.data.status = JobStatus::Success;
                        node.node_type = "job-success".to_string();
                    }
                }
            }

            // Mark current as running
            if let Some(node) = s.nodes.iter_mut().find(|n| n.id == job_order[step]) {
                node.data.status = JobStatus::Running;
                node.node_type = "job-running".to_string();
            }
            // Handle parallel (lint and test)
            if step == 0 {
                if let Some(node) = s.nodes.iter_mut().find(|n| n.id == "test") {
                    node.data.status = JobStatus::Running;
                    node.node_type = "job-running".to_string();
                }
            }

            // Update edges - collect node statuses first to avoid borrow conflict
            let node_statuses: HashMap<String, JobStatus> = s.nodes
                .iter()
                .map(|n| (n.id.clone(), n.data.status.clone()))
                .collect();

            for edge in &mut s.edges {
                if let Some(status) = node_statuses.get(&edge.source) {
                    edge.animated = *status == JobStatus::Success;
                    if *status == JobStatus::Success {
                        edge.stroke = "#22c55e".to_string();
                    }
                }
            }

            current_step.set(step + 1);
        } else {
            // Mark last as success
            let mut s = state.write();
            if let Some(node) = s.nodes.iter_mut().find(|n| n.id == "deploy-prod") {
                node.data.status = JobStatus::Success;
                node.node_type = "job-success".to_string();
            }
            for edge in &mut s.edges {
                edge.animated = false;
                edge.stroke = "#22c55e".to_string();
            }
        }
    };

    rsx! {
        style { "{FLOW_STYLES}" }
        style { "{WORKFLOW_STYLES}" }

        div {
            class: "app-container",

            div {
                class: "header",
                h1 { "CI/CD Pipeline Visualization" }
                p { "Workflow parsed from YAML configuration" }
            }

            div {
                class: "toolbar",
                button {
                    onclick: run_workflow,
                    "Reset Workflow"
                }
                button {
                    onclick: step_forward,
                    "Step Forward"
                }
                span {
                    class: "status",
                    "Step: {current_step} / 5"
                }
            }

            div {
                class: "main-content",

                div {
                    class: "flow-container",
                    Flow {
                        state: state,
                        on_node_click: move |node_id: String| {
                            // Could show job details in a sidebar
                            web_sys::console::log_1(&format!("Clicked job: {}", node_id).into());
                        },
                    }
                }

                div {
                    class: "yaml-preview",
                    h3 { "Workflow YAML" }
                    pre {
                        code { "{SAMPLE_WORKFLOW}" }
                    }
                }
            }

            div {
                class: "legend",
                div { class: "legend-item",
                    span { class: "legend-dot job-pending" }
                    "Pending"
                }
                div { class: "legend-item",
                    span { class: "legend-dot job-running" }
                    "Running"
                }
                div { class: "legend-item",
                    span { class: "legend-dot job-success" }
                    "Success"
                }
                div { class: "legend-item",
                    span { class: "legend-dot job-failed" }
                    "Failed"
                }
            }
        }
    }
}

const WORKFLOW_STYLES: &str = r#"
* {
    box-sizing: border-box;
}

body, html, #main {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0d1117;
    color: #c9d1d9;
}

.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
}

.header {
    padding: 16px 24px;
    background: #161b22;
    border-bottom: 1px solid #30363d;
}

.header h1 {
    margin: 0 0 4px 0;
    font-size: 20px;
    font-weight: 600;
}

.header p {
    margin: 0;
    font-size: 14px;
    color: #8b949e;
}

.toolbar {
    padding: 12px 24px;
    background: #161b22;
    border-bottom: 1px solid #30363d;
    display: flex;
    gap: 12px;
    align-items: center;
}

.toolbar button {
    padding: 8px 16px;
    border: 1px solid #30363d;
    border-radius: 6px;
    background: #21262d;
    color: #c9d1d9;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
}

.toolbar button:hover {
    background: #30363d;
    border-color: #8b949e;
}

.toolbar .status {
    margin-left: auto;
    font-size: 14px;
    color: #8b949e;
}

.main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
}

.flow-container {
    flex: 1;
    background: #0d1117;
}

.yaml-preview {
    width: 400px;
    background: #161b22;
    border-left: 1px solid #30363d;
    padding: 16px;
    overflow: auto;
}

.yaml-preview h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: #8b949e;
}

.yaml-preview pre {
    margin: 0;
    padding: 12px;
    background: #0d1117;
    border-radius: 6px;
    font-size: 12px;
    line-height: 1.5;
    overflow: auto;
}

.yaml-preview code {
    color: #c9d1d9;
}

.legend {
    padding: 12px 24px;
    background: #161b22;
    border-top: 1px solid #30363d;
    display: flex;
    gap: 24px;
}

.legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #8b949e;
}

.legend-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
}

/* Flow container override */
.dioxus-flow-container {
    background-color: #0d1117 !important;
    background-image: radial-gradient(#30363d 1px, transparent 1px) !important;
}

/* Job node styles */
.dioxus-flow-node {
    min-width: 180px;
    padding: 12px 16px;
    border-radius: 6px;
    border: 1px solid #30363d;
    background: #161b22;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    font-size: 14px;
    color: #c9d1d9;
}

.dioxus-flow-node-job-pending {
    border-left: 3px solid #8b949e;
}

.dioxus-flow-node-job-running {
    border-left: 3px solid #f0883e;
    animation: pulse 2s infinite;
}

.dioxus-flow-node-job-success {
    border-left: 3px solid #3fb950;
}

.dioxus-flow-node-job-failed {
    border-left: 3px solid #f85149;
}

.dioxus-flow-node-job-skipped {
    border-left: 3px solid #6e7681;
    opacity: 0.6;
}

.job-pending { background: #8b949e; }
.job-running { background: #f0883e; }
.job-success { background: #3fb950; }
.job-failed { background: #f85149; }
.job-skipped { background: #6e7681; }

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
}

/* Handle styles */
.dioxus-flow-handle {
    background: #58a6ff !important;
    border-color: #0d1117 !important;
}

/* Edge styles */
.dioxus-flow-edge-path {
    stroke: #30363d;
}

.dioxus-flow-edge-selected .dioxus-flow-edge-path {
    stroke: #58a6ff;
}

.dioxus-flow-edge-animated .dioxus-flow-edge-path {
    stroke: #f0883e;
}
"#;

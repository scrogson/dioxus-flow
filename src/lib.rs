//! # dioxus-flow
//!
//! An xyflow-like library for building node-based editors in Dioxus.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use dioxus_flow::prelude::*;
//!
//! fn App() -> Element {
//!     let nodes = use_signal(|| vec![
//!         Node::new("1", 100.0, 100.0).with_data("Node 1"),
//!         Node::new("2", 300.0, 200.0).with_data("Node 2"),
//!     ]);
//!
//!     let edges = use_signal(|| vec![
//!         Edge::new("e1", "1", "2"),
//!     ]);
//!
//!     rsx! {
//!         Flow {
//!             nodes: nodes,
//!             edges: edges,
//!         }
//!     }
//! }
//! ```

pub mod components;
pub mod hooks;
pub mod types;
pub mod utils;

pub mod prelude {
    //! Convenient re-exports for common usage.
    pub use crate::components::edge::EdgeComponent;
    pub use crate::components::flow::Flow;
    pub use crate::components::handle::{Handle, HandleType};
    pub use crate::components::node::NodeComponent;
    pub use crate::hooks::{use_flow, FlowState};
    pub use crate::types::*;
}

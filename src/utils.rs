//! Utility functions for dioxus-flow.

use crate::types::{EdgeType, Position};

/// Calculate the path for a bezier edge.
pub fn get_bezier_path(
    source: Position,
    target: Position,
    source_position: crate::types::HandlePosition,
    target_position: crate::types::HandlePosition,
) -> String {
    let (sx, sy) = (source.x, source.y);
    let (tx, ty) = (target.x, target.y);

    // Calculate control point offsets based on handle positions
    let offset = ((tx - sx).abs() + (ty - sy).abs()) / 3.0;
    let offset = offset.max(50.0);

    let (sc_x, sc_y) = get_control_point_offset(source_position, offset);
    let (tc_x, tc_y) = get_control_point_offset(target_position, offset);

    format!(
        "M {sx},{sy} C {},{} {},{} {tx},{ty}",
        sx + sc_x,
        sy + sc_y,
        tx + tc_x,
        ty + tc_y
    )
}

/// Get control point offset based on handle position.
fn get_control_point_offset(
    position: crate::types::HandlePosition,
    offset: f64,
) -> (f64, f64) {
    use crate::types::HandlePosition;
    match position {
        HandlePosition::Top => (0.0, -offset),
        HandlePosition::Right => (offset, 0.0),
        HandlePosition::Bottom => (0.0, offset),
        HandlePosition::Left => (-offset, 0.0),
    }
}

/// Calculate the path for a straight edge.
pub fn get_straight_path(source: Position, target: Position) -> String {
    format!("M {},{} L {},{}", source.x, source.y, target.x, target.y)
}

/// Calculate the path for a step edge.
pub fn get_step_path(
    source: Position,
    target: Position,
    source_position: crate::types::HandlePosition,
    _target_position: crate::types::HandlePosition,
) -> String {
    use crate::types::HandlePosition;

    let (sx, sy) = (source.x, source.y);
    let (tx, ty) = (target.x, target.y);

    match source_position {
        HandlePosition::Top | HandlePosition::Bottom => {
            let mid_y = (sy + ty) / 2.0;
            format!("M {sx},{sy} L {sx},{mid_y} L {tx},{mid_y} L {tx},{ty}")
        }
        HandlePosition::Left | HandlePosition::Right => {
            let mid_x = (sx + tx) / 2.0;
            format!("M {sx},{sy} L {mid_x},{sy} L {mid_x},{ty} L {tx},{ty}")
        }
    }
}

/// Calculate the path for a smooth step edge.
pub fn get_smooth_step_path(
    source: Position,
    target: Position,
    source_position: crate::types::HandlePosition,
    _target_position: crate::types::HandlePosition,
    border_radius: f64,
) -> String {
    use crate::types::HandlePosition;

    let (sx, sy) = (source.x, source.y);
    let (tx, ty) = (target.x, target.y);
    let r = border_radius;

    match source_position {
        HandlePosition::Top | HandlePosition::Bottom => {
            let mid_y = (sy + ty) / 2.0;
            let dir_y = if ty > sy { 1.0 } else { -1.0 };
            let dir_x = if tx > sx { 1.0 } else { -1.0 };

            format!(
                "M {sx},{sy} L {sx},{} Q {sx},{mid_y} {},{mid_y} L {},{mid_y} Q {tx},{mid_y} {tx},{} L {tx},{ty}",
                mid_y - r * dir_y,
                sx + r * dir_x,
                tx - r * dir_x,
                mid_y + r * dir_y
            )
        }
        HandlePosition::Left | HandlePosition::Right => {
            let mid_x = (sx + tx) / 2.0;
            let dir_x = if tx > sx { 1.0 } else { -1.0 };
            let dir_y = if ty > sy { 1.0 } else { -1.0 };

            format!(
                "M {sx},{sy} L {},{sy} Q {mid_x},{sy} {mid_x},{} L {mid_x},{} Q {mid_x},{ty} {},{ty} L {tx},{ty}",
                mid_x - r * dir_x,
                sy + r * dir_y,
                ty - r * dir_y,
                mid_x + r * dir_x
            )
        }
    }
}

/// Get the edge path based on edge type.
pub fn get_edge_path(
    edge_type: EdgeType,
    source: Position,
    target: Position,
    source_position: crate::types::HandlePosition,
    target_position: crate::types::HandlePosition,
) -> String {
    match edge_type {
        EdgeType::Bezier => get_bezier_path(source, target, source_position, target_position),
        EdgeType::Straight => get_straight_path(source, target),
        EdgeType::Step => get_step_path(source, target, source_position, target_position),
        EdgeType::SmoothStep => {
            get_smooth_step_path(source, target, source_position, target_position, 5.0)
        }
    }
}

/// Clamp a value between min and max.
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Calculate distance between two positions.
pub fn distance(a: Position, b: Position) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

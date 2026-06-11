use crate::{
    config::LayoutConfig,
    input::types::{Point, Rect},
    protocol::messages::LayoutDirection,
};

pub fn is_handoff_edge_hit(layout: &LayoutConfig, bounds: Rect, pointer: Point) -> bool {
    let left = bounds.x;
    let right = bounds.x + bounds.width - 1.0;
    let top = bounds.y;
    let bottom = bounds.y + bounds.height - 1.0;
    let guard = f64::from(layout.corner_guard_px);
    let thickness = f64::from(layout.edge_thickness_px.max(1));

    match layout.direction {
        LayoutDirection::Left => {
            pointer.x <= left + thickness && pointer.y >= top + guard && pointer.y <= bottom - guard
        }
        LayoutDirection::Right => {
            pointer.x >= right - thickness
                && pointer.y >= top + guard
                && pointer.y <= bottom - guard
        }
        LayoutDirection::Top => {
            pointer.y <= top + thickness && pointer.x >= left + guard && pointer.x <= right - guard
        }
        LayoutDirection::Bottom => {
            pointer.y >= bottom - thickness
                && pointer.x >= left + guard
                && pointer.x <= right - guard
        }
    }
}

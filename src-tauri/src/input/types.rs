use serde::{Deserialize, Serialize};

use crate::protocol::messages::{MouseButton, TimestampMs};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LocalMouseEvent {
    Move {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        ts_ms: TimestampMs,
    },
    Down {
        button: MouseButton,
        x: f64,
        y: f64,
        ts_ms: TimestampMs,
    },
    Up {
        button: MouseButton,
        x: f64,
        y: f64,
        ts_ms: TimestampMs,
    },
    Wheel {
        dx: f64,
        dy: f64,
        ts_ms: TimestampMs,
    },
}


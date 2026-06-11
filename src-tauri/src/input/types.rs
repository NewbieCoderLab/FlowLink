use std::thread::JoinHandle;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub struct DisplayInfo {
    pub id: u64,
    pub bounds: Rect,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenTopology {
    pub displays: Vec<DisplayInfo>,
    pub virtual_bounds: Rect,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemoteMouseEvent {
    Move { dx: f64, dy: f64 },
    MoveTo { x: f64, y: f64 },
    Down { button: MouseButton, x: f64, y: f64 },
    Up { button: MouseButton, x: f64, y: f64 },
    Wheel { dx: f64, dy: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKind {
    Accessibility,
    InputMonitoring,
    WindowsInput,
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("input platform is unsupported")]
    Unsupported,
    #[error("permission is not granted: {0}")]
    PermissionDenied(&'static str),
    #[error("capture is already running")]
    CaptureAlreadyRunning,
    #[error("platform error: {0}")]
    Platform(String),
}

pub type InputResult<T> = Result<T, InputError>;

#[derive(Debug)]
pub struct CaptureHandle {
    join_handle: Option<JoinHandle<()>>,
}

impl CaptureHandle {
    pub fn detached(join_handle: JoinHandle<()>) -> Self {
        Self {
            join_handle: Some(join_handle),
        }
    }

    pub fn noop() -> Self {
        Self { join_handle: None }
    }

    pub fn is_running(&self) -> bool {
        self.join_handle.is_some()
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        // Platform capture threads own OS run loops. S1 keeps them detached;
        // later session work will add explicit shutdown handles.
        let _ = self.join_handle.take();
    }
}

pub fn empty_screen_topology() -> ScreenTopology {
    ScreenTopology {
        displays: Vec::new(),
        virtual_bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    }
}

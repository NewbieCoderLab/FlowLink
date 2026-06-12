use std::{thread::JoinHandle, time::Duration};

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

pub struct CaptureHandle {
    join_handle: Option<JoinHandle<()>>,
    shutdown: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl CaptureHandle {
    pub fn new(
        join_handle: JoinHandle<()>,
        shutdown: impl FnOnce() + Send + Sync + 'static,
    ) -> Self {
        Self {
            join_handle: Some(join_handle),
            shutdown: Some(Box::new(shutdown)),
        }
    }

    pub fn detached(join_handle: JoinHandle<()>) -> Self {
        Self {
            join_handle: Some(join_handle),
            shutdown: None,
        }
    }

    pub fn noop() -> Self {
        Self {
            join_handle: None,
            shutdown: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.join_handle.is_some()
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown();
        }
        if let Some(join_handle) = self.join_handle.take() {
            let deadline = std::time::Instant::now() + Duration::from_millis(250);
            while !join_handle.is_finished() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if join_handle.is_finished() {
                let _ = join_handle.join();
            }
        }
    }
}

impl std::fmt::Debug for CaptureHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CaptureHandle")
            .field("is_running", &self.is_running())
            .finish()
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

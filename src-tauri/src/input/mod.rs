pub mod edge;
pub mod macos;
pub mod noop;
pub mod types;
pub mod windows;

use tokio::sync::mpsc;

use crate::platform::PermissionStatus;

use types::{
    CaptureHandle, InputResult, LocalMouseEvent, PermissionKind, Point, RemoteMouseEvent,
    ScreenTopology,
};

pub trait InputPlatform: Send + Sync {
    fn permissions(&self) -> PermissionStatus;
    fn request_permissions(&self, kind: PermissionKind) -> InputResult<()>;
    fn screen_topology(&self) -> InputResult<ScreenTopology>;
    fn start_capture(&self, tx: mpsc::Sender<LocalMouseEvent>) -> InputResult<CaptureHandle>;
    fn inject(&self, event: RemoteMouseEvent) -> InputResult<()>;
    fn warp_cursor(&self, position: Point) -> InputResult<()>;
}

pub fn platform_input() -> Box<dyn InputPlatform> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacInputPlatform::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WinInputPlatform::new())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Box::new(noop::NoopInputPlatform)
    }
}

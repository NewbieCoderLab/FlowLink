use tokio::sync::mpsc;

use crate::platform::PermissionStatus;

use super::{
    types::{
        empty_screen_topology, CaptureHandle, InputError, InputResult, LocalMouseEvent,
        PermissionKind, Point, RemoteMouseEvent, ScreenTopology,
    },
    InputPlatform,
};

pub fn platform_name() -> &'static str {
    "noop"
}

pub struct NoopInputPlatform;

impl InputPlatform for NoopInputPlatform {
    fn permissions(&self) -> PermissionStatus {
        PermissionStatus::unsupported()
    }

    fn request_permissions(&self, _kind: PermissionKind) -> InputResult<()> {
        Err(InputError::Unsupported)
    }

    fn screen_topology(&self) -> InputResult<ScreenTopology> {
        Ok(empty_screen_topology())
    }

    fn start_capture(&self, _tx: mpsc::Sender<LocalMouseEvent>) -> InputResult<CaptureHandle> {
        Err(InputError::Unsupported)
    }

    fn inject(&self, _event: RemoteMouseEvent) -> InputResult<()> {
        Err(InputError::Unsupported)
    }

    fn warp_cursor(&self, _position: Point) -> InputResult<()> {
        Err(InputError::Unsupported)
    }
}

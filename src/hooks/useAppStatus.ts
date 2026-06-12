import { useEffect, useState } from "react";
import {
  getAppStatus,
  listenDeviceDiscoveryUpdates,
  listenPermissionUpdates
} from "../app/tauri";
import type { UiAppStatus } from "../app/types";

export function useAppStatus() {
  const [status, setStatus] = useState<UiAppStatus | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    const nextStatus = await getAppStatus();
    setStatus(nextStatus);
    setLoading(false);
  };

  useEffect(() => {
    void refresh();
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlistenPermissionUpdates: (() => void) | null = null;
    let unlistenDeviceDiscoveryUpdates: (() => void) | null = null;
    const refreshIfActive = () => {
      if (!disposed) {
        void refresh();
      }
    };
    const refreshWhenVisible = () => {
      if (document.visibilityState === "visible") {
        refreshIfActive();
      }
    };

    window.addEventListener("focus", refreshIfActive);
    document.addEventListener("visibilitychange", refreshWhenVisible);
    void listenPermissionUpdates(refreshIfActive).then((unlisten) => {
      if (disposed) {
        unlisten();
      } else {
        unlistenPermissionUpdates = unlisten;
      }
    });
    void listenDeviceDiscoveryUpdates(refreshIfActive).then((unlisten) => {
      if (disposed) {
        unlisten();
      } else {
        unlistenDeviceDiscoveryUpdates = unlisten;
      }
    });

    return () => {
      disposed = true;
      window.removeEventListener("focus", refreshIfActive);
      document.removeEventListener("visibilitychange", refreshWhenVisible);
      unlistenPermissionUpdates?.();
      unlistenDeviceDiscoveryUpdates?.();
    };
  }, []);

  return { status, loading, refresh };
}

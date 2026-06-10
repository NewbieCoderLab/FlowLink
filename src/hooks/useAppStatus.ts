import { useEffect, useState } from "react";
import { getAppStatus } from "../app/tauri";
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

  return { status, loading, refresh };
}


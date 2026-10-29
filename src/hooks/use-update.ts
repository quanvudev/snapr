import { useEffect, useState } from "react";
import { api, type UpdateStatus } from "@/lib/api";

const IDLE: UpdateStatus = { state: "idle", version: null, done: 0, total: 0, error: null };

export function useUpdate() {
  const [status, setStatus] = useState<UpdateStatus>(IDLE);

  useEffect(() => {
    const refresh = () => api.updateStatus().then(setStatus);
    refresh();
    const subs = [api.onSettingsOpen(refresh), api.onUpdateStatus(setStatus)];
    return () => {
      subs.forEach((u) => u.then((f) => f()));
    };
  }, []);

  return {
    status,
    check: () => api.updateCheck(),
    install: () => api.updateInstall(),
  };
}

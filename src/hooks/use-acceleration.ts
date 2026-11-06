import { useCallback, useEffect, useState } from "react";
import { api, type AccelerationProgress, type AccelerationStatus, type GpuRuntime } from "@/lib/api";

const EMPTY: AccelerationStatus = {
  adapter: null,
  recommended: "cpu",
  resolved: "cpu",
  loaded: null,
  restart_needed: false,
  runtimes: [],
  downloading: null,
  error: null,
};

export function useAcceleration() {
  const [status, setStatus] = useState<AccelerationStatus>(EMPTY);
  const [progress, setProgress] = useState<AccelerationProgress | null>(null);
  const [error, setError] = useState<{ runtime: GpuRuntime; message: string } | null>(null);
  const [restartError, setRestartError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.accelerationStatus();
    setStatus(s);
    setProgress(s.downloading);
    return s;
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onAccelerationProgress(setProgress),
      api.onAccelerationDone(async ({ runtime, error }) => {
        setProgress(null);
        setError(error ? { runtime, message: error } : null);
        await refresh();
      }),
    ];
    return () => {
      subs.forEach((u) => u.then((f) => f()));
    };
  }, [refresh]);

  const download = async (runtime: GpuRuntime) => {
    setError(null);
    setProgress({ runtime, done: 0, total: 0 });
    try {
      await api.accelerationDownload(runtime);
    } catch (e) {
      setProgress(null);
      setError({ runtime, message: String(e) });
    }
  };

  const cancel = () => api.accelerationCancel();

  const remove = async (runtime: GpuRuntime) => {
    try {
      await api.accelerationDelete(runtime);
    } catch (e) {
      setError({ runtime, message: String(e) });
    }
    await refresh();
  };

  const restart = async () => {
    setRestartError(null);
    try {
      await api.accelerationRestart();
    } catch (e) {
      setRestartError(String(e));
    }
  };

  return { status, progress, error, restartError, refresh, download, cancel, remove, restart };
}

import { useCallback, useEffect, useState } from "react";
import { api, type UpscaleProgress, type UpscaleStatus } from "@/lib/api";

const EMPTY: UpscaleStatus = { ready: false, bytes: 0, downloading: null };

export function useUpscale() {
  const [status, setStatus] = useState<UpscaleStatus>(EMPTY);
  const [progress, setProgress] = useState<UpscaleProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.upscaleStatus();
    setStatus(s);
    setProgress(s.downloading);
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onUpscaleProgress(setProgress),
      api.onUpscaleDone(async ({ error }) => {
        setProgress(null);
        setError(error);
        await refresh();
      }),
    ];
    return () => {
      subs.forEach((u) => u.then((f) => f()));
    };
  }, [refresh]);

  const download = async () => {
    setError(null);
    setProgress({ done: 0, total: 0 });
    try {
      await api.upscaleDownload();
    } catch (e) {
      setProgress(null);
      setError(String(e));
    }
  };

  const cancel = () => api.upscaleCancel();

  const remove = async () => {
    try {
      await api.upscaleDelete();
    } catch (e) {
      setError(String(e));
    }
    await refresh();
  };

  return { status, progress, error, download, cancel, remove };
}

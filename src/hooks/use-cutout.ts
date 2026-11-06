import { useCallback, useEffect, useState } from "react";
import { api, type CutoutProgress, type CutoutStatus } from "@/lib/api";

const EMPTY: CutoutStatus = { ready: false, bytes: 0, downloading: null };

export function useCutout() {
  const [status, setStatus] = useState<CutoutStatus>(EMPTY);
  const [progress, setProgress] = useState<CutoutProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.cutoutStatus();
    setStatus(s);
    setProgress(s.downloading);
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onCutoutProgress(setProgress),
      api.onCutoutDone(async ({ error }) => {
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
      await api.cutoutDownload();
    } catch (e) {
      setProgress(null);
      setError(String(e));
    }
  };

  const cancel = () => api.cutoutCancel();

  const remove = async () => {
    try {
      await api.cutoutDelete();
    } catch (e) {
      setError(String(e));
    }
    await refresh();
  };

  return { status, progress, error, download, cancel, remove };
}

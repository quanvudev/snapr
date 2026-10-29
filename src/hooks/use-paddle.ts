import { useCallback, useEffect, useState } from "react";
import { api, type PaddleProgress, type PaddleSize, type PaddleStatus } from "@/lib/api";

const EMPTY: PaddleStatus = { ready: [], bytes: { tiny: 0, small: 0, medium: 0 }, downloading: null, runtime: false };

export function usePaddle(onReady: () => void) {
  const [status, setStatus] = useState<PaddleStatus>(EMPTY);
  const [progress, setProgress] = useState<PaddleProgress | null>(null);
  const [error, setError] = useState<{ size: PaddleSize; message: string } | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.paddleStatus();
    setStatus(s);
    setProgress(s.downloading);
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onPaddleProgress(setProgress),
      api.onPaddleDone(async ({ size, error }) => {
        setProgress(null);
        setError(error ? { size, message: error } : null);
        await refresh();
        if (!error) onReady();
      }),
    ];
    return () => {
      subs.forEach((u) => u.then((f) => f()));
    };
  }, [refresh, onReady]);

  const download = async (size: PaddleSize) => {
    setError(null);
    setProgress({ size, done: 0, total: 0 });
    try {
      await api.paddleDownload(size);
    } catch (e) {
      setProgress(null);
      setError({ size, message: String(e) });
    }
  };

  const cancel = () => api.paddleCancel();

  const remove = async (size: PaddleSize) => {
    try {
      await api.paddleDelete(size);
    } catch (e) {
      setError({ size, message: String(e) });
    }
    await refresh();
    onReady();
  };

  return { status, progress, error, download, cancel, remove };
}

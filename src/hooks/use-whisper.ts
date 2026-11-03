import { useCallback, useEffect, useState } from "react";
import { api, type WhisperModel, type WhisperProgress, type WhisperStatus } from "@/lib/api";

const EMPTY: WhisperStatus = { ready: [], bytes: { base: 0, small: 0, turbo: 0 }, downloading: null };

export function useWhisper() {
  const [status, setStatus] = useState<WhisperStatus>(EMPTY);
  const [progress, setProgress] = useState<WhisperProgress | null>(null);
  const [error, setError] = useState<{ model: WhisperModel; message: string } | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.whisperStatus();
    setStatus(s);
    setProgress(s.downloading);
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onWhisperProgress(setProgress),
      api.onWhisperDone(async ({ model, error }) => {
        setProgress(null);
        setError(error ? { model, message: error } : null);
        await refresh();
      }),
    ];
    return () => {
      subs.forEach((u) => u.then((f) => f()));
    };
  }, [refresh]);

  const download = async (model: WhisperModel) => {
    setError(null);
    setProgress({ model, done: 0, total: 0 });
    try {
      await api.whisperDownload(model);
    } catch (e) {
      setProgress(null);
      setError({ model, message: String(e) });
    }
  };

  const cancel = () => api.whisperCancel();

  const remove = async (model: WhisperModel) => {
    try {
      await api.whisperDelete(model);
    } catch (e) {
      setError({ model, message: String(e) });
    }
    await refresh();
  };

  return { status, progress, error, download, cancel, remove };
}

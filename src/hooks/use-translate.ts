import { useCallback, useEffect, useState } from "react";
import { api, type TranslateProgress, type TranslateStatus } from "@/lib/api";

const EMPTY: TranslateStatus = { ready: false, bytes: 0, downloading: null };

export function useTranslate() {
  const [status, setStatus] = useState<TranslateStatus>(EMPTY);
  const [progress, setProgress] = useState<TranslateProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.translateStatus();
    setStatus(s);
    setProgress(s.downloading);
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onTranslateProgress(setProgress),
      api.onTranslateDone(async ({ error }) => {
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
      await api.translateDownload();
    } catch (e) {
      setProgress(null);
      setError(String(e));
    }
  };

  const cancel = () => api.translateCancel();

  const remove = async () => {
    try {
      await api.translateDelete();
    } catch (e) {
      setError(String(e));
    }
    await refresh();
  };

  return { status, progress, error, download, cancel, remove };
}

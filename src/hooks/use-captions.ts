import { useCallback, useEffect, useState } from "react";
import { api, type CaptionLanguage, type CaptionsProgress, type CaptionsStatus } from "@/lib/api";

const EMPTY: CaptionsStatus = { models: [], downloading: null };

export function useCaptions() {
  const [status, setStatus] = useState<CaptionsStatus>(EMPTY);
  const [progress, setProgress] = useState<CaptionsProgress | null>(null);
  const [error, setError] = useState<{ language: CaptionLanguage; message: string } | null>(null);

  const refresh = useCallback(async () => {
    const s = await api.captionsStatus();
    setStatus(s);
    setProgress(s.downloading);
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      api.onSettingsOpen(() => refresh()),
      api.onCaptionsProgress(setProgress),
      api.onCaptionsDone(async ({ language, error }) => {
        setProgress(null);
        setError(error ? { language, message: error } : null);
        await refresh();
      }),
    ];
    return () => {
      subs.forEach((u) => u.then((f) => f()));
    };
  }, [refresh]);

  const download = async (language: CaptionLanguage) => {
    setError(null);
    setProgress({ language, done: 0, total: 0 });
    try {
      await api.captionsDownload(language);
    } catch (e) {
      setProgress(null);
      setError({ language, message: String(e) });
    }
  };

  const cancel = () => api.captionsCancel();

  const remove = async (language: CaptionLanguage) => {
    try {
      await api.captionsDelete(language);
    } catch (e) {
      setError({ language, message: String(e) });
    }
    await refresh();
  };

  return { status, progress, error, download, cancel, remove };
}

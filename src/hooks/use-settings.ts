import { useCallback, useEffect, useRef, useState } from "react";
import { api, type Config, type Row } from "@/lib/api";
import { SLOTS } from "@/lib/hotkeys";
import { T } from "@/lib/i18n";
import { useHotkeyRecorder } from "./use-hotkey-recorder";

export type View = "splash" | "setup" | "settings";
export type SectionId = "gallery" | "settings";

export function useSettings() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [autostart, setAutostart] = useState(false);
  const [dir, setDir] = useState("");
  const [rows, setRows] = useState<Row[]>([]);
  const [ocr, setOcr] = useState(false);
  const [failed, setFailed] = useState(false);
  const [view, setView] = useState<View>("settings");
  const [section, setSection] = useState<SectionId>("settings");
  const [version, setVersion] = useState("");
  const firstLoad = useRef(true);
  const timer = useRef<number>(0);

  const { recording, setRecording } = useHotkeyRecorder((slot, key) => {
    if (cfg) update({ ...cfg, hotkeys: { ...cfg.hotkeys, [SLOTS[slot]]: key } });
  }, view !== "splash");

  const load = useCallback(
    async (fromEvent: boolean, openSection?: string) => {
      const s = await api.load();
      setCfg(s.config);
      setAutostart(s.autostart);
      setDir(s.dir);
      setOcr(s.ocr_available);
      setRecording(null);
      setFailed(false);
      if (fromEvent) {
        clearTimeout(timer.current);
        setSection((openSection as SectionId) || "settings");
        setView(s.first_run ? "setup" : "settings");
      } else if (firstLoad.current) {
        firstLoad.current = false;
        if (s.first_run) {
          setView("splash");
          timer.current = window.setTimeout(() => {
            api.openSettings().then(() => setView("setup"));
          }, 1000);
        } else {
          setView("settings");
        }
      }
    },
    [setRecording],
  );

  useEffect(() => {
    api.version().then(setVersion);
    load(false);
    const un = api.onSettingsOpen((s) => load(true, s));
    return () => {
      un.then((f) => f());
    };
  }, [load]);

  const outputDir = cfg?.output_dir;
  useEffect(() => {
    if (outputDir !== undefined) api.resolveDir(outputDir).then(setDir);
  }, [outputDir]);

  const hotkeys = cfg?.hotkeys;
  const textActive = !!cfg?.text_capture && ocr;
  useEffect(() => {
    if (hotkeys) api.check(hotkeys, textActive).then(setRows);
  }, [hotkeys, textActive]);

  const persist = async (next: Config, nextAutostart: boolean) => {
    setCfg(next);
    setAutostart(nextAutostart);
    setFailed(false);
    const checked = await api.check(next.hotkeys, next.text_capture && ocr);
    setRows(checked);
    if (checked.some((r) => r.error)) return;
    try {
      await api.save(next, nextAutostart);
    } catch {
      setFailed(true);
    }
  };

  const update = (next: Config, nextAutostart = autostart) => {
    if (view === "setup") {
      setCfg(next);
      setAutostart(nextAutostart);
    } else {
      persist(next, nextAutostart);
    }
  };

  const start = async () => {
    if (!cfg) return;
    try {
      await api.save(cfg, autostart);
      api.hide();
    } catch {
      setFailed(true);
    }
  };

  if (!cfg) return null;

  return {
    cfg,
    autostart,
    dir,
    ocr,
    textActive,
    rows,
    failed,
    view,
    section,
    setSection,
    version,
    recording,
    setRecording,
    t: T[cfg.language],
    update,
    start,
    reset: async () => {
      const d = await api.defaults();
      setRecording(null);
      update({ ...d, language: cfg.language });
    },
  };
}

export type SettingsValue = NonNullable<ReturnType<typeof useSettings>>;

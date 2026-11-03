import { useCallback, useEffect, useRef, useState } from "react";
import { api, type Config, type Row } from "@/lib/api";
import { defaultCombos, SLOTS } from "@/lib/hotkeys";
import { T } from "@/lib/i18n";
import { useHotkeyRecorder } from "./use-hotkey-recorder";
import { usePaddle } from "./use-paddle";
import { useUpdate } from "./use-update";
import { useWhisper } from "./use-whisper";

export type View = "splash" | "setup" | "settings";
export type SectionId = "gallery" | "general" | "capture" | "text" | "recording" | "hotkeys" | "ring" | "models" | "about";

const LEGACY_SECTIONS: Record<string, SectionId> = { settings: "general" };

export function useSettings() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [autostart, setAutostart] = useState(false);
  const [dir, setDir] = useState("");
  const [rows, setRows] = useState<Row[]>([]);
  const [ocr, setOcr] = useState(false);
  const [windowsOcr, setWindowsOcr] = useState(false);
  const [failed, setFailed] = useState(false);
  const [view, setView] = useState<View>("settings");
  const [section, setSection] = useState<SectionId>("general");
  const [version, setVersion] = useState("");
  const firstLoad = useRef(true);
  const timer = useRef<number>(0);

  const { recording, setRecording } = useHotkeyRecorder((slot, key) => {
    if (!cfg) return;
    const { hotkeys } = cfg;
    const next =
      hotkeys.mode === "fixed"
        ? { ...hotkeys, [SLOTS[slot]]: key }
        : { ...hotkeys, manual: hotkeys.manual.map((c, i) => (i === slot ? key : c)) };
    update({ ...cfg, hotkeys: next });
  }, view !== "splash", cfg?.hotkeys.mode ?? "fixed");

  const load = useCallback(
    async (fromEvent: boolean, openSection?: string) => {
      const s = await api.load();
      setCfg(s.config);
      setAutostart(s.autostart);
      setDir(s.dir);
      setOcr(s.ocr_available);
      setWindowsOcr(s.windows_ocr);
      setRecording(null);
      setFailed(false);
      if (fromEvent) {
        clearTimeout(timer.current);
        setSection(LEGACY_SECTIONS[openSection ?? ""] ?? ((openSection as SectionId) || "general"));
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

  const reloadOcr = useCallback(async () => {
    const s = await api.load();
    setOcr(s.ocr_available);
    setWindowsOcr(s.windows_ocr);
  }, []);
  const paddle = usePaddle(reloadOcr);
  const whisper = useWhisper();
  const updater = useUpdate();

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
      const switched = cfg?.ocr_engine !== next.ocr_engine || cfg?.paddle_model !== next.paddle_model;
      if (switched && next.ocr_engine === "paddle" && !paddle.status.ready.includes(next.paddle_model) && paddle.progress?.size !== next.paddle_model) {
        paddle.download(next.paddle_model);
      }
      const whisperSwitched =
        (!cfg?.record_transcribe && next.record_transcribe) || cfg?.whisper_model !== next.whisper_model;
      if (
        whisperSwitched &&
        !whisper.status.ready.includes(next.whisper_model) &&
        whisper.progress?.model !== next.whisper_model
      ) {
        whisper.download(next.whisper_model);
      }
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
    windowsOcr,
    paddle,
    whisper,
    updater,
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
    restoreHotkeys: async () => {
      const { hotkeys: d } = await api.defaults();
      setRecording(null);
      const { hotkeys } = cfg;
      const next =
        hotkeys.mode === "fixed"
          ? { ...hotkeys, modifiers: d.modifiers, ...Object.fromEntries(SLOTS.map((s) => [s, d[s]])) }
          : { ...hotkeys, manual: defaultCombos(d) };
      update({ ...cfg, hotkeys: next });
    },
  };
}

export type SettingsValue = NonNullable<ReturnType<typeof useSettings>>;

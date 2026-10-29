import { useEffect, useRef } from "react";
import { usePanel } from "@/hooks/use-panel";
import { useTheme } from "@/hooks/use-theme";
import { api } from "@/lib/api";
import "@/App.css";
import { RecordingPanel } from "./recording-panel";
import { SetupPanel } from "./setup-panel";

export function PanelApp() {
  const { state, level, cfg, t } = usePanel();
  useTheme(cfg?.theme);

  const root = useRef<HTMLDivElement>(null);
  const ready = !!state && !!cfg;

  useEffect(() => {
    for (const el of [document.documentElement, document.body, document.getElementById("root")]) {
      if (!el) continue;
      el.style.background = "transparent";
      el.style.overflow = "hidden";
    }
  }, []);

  useEffect(() => {
    const el = root.current;
    if (!el) return;
    const fit = () => {
      const r = el.getBoundingClientRect();
      if (r.width > 0 && r.height > 0) api.recordPanelFit(Math.ceil(r.width), Math.ceil(r.height));
    };
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    fit();
    return () => observer.disconnect();
  }, [ready, state?.phase]);

  if (!state || !cfg) return null;
  const compact = state.phase === "recording" || state.phase === "countdown";
  if (state.phase !== "setup" && !compact) return null;

  return (
    <div ref={root} className="w-fit">
      {compact ? <RecordingPanel state={state} t={t} /> : <SetupPanel state={state} level={level} t={t} />}
    </div>
  );
}

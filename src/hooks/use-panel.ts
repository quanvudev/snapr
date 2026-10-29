import { useEffect, useState } from "react";
import { api, type Config, type PanelState } from "@/lib/api";
import { T } from "@/lib/i18n";

export function usePanel() {
  const [state, setState] = useState<PanelState | null>(null);
  const [level, setLevel] = useState(0);
  const [cfg, setCfg] = useState<Config | null>(null);

  useEffect(() => {
    let alive = true;
    const unlisten = [api.onRecordPanel(setState), api.onRecordLevel(setLevel)];
    api.recordPanelState().then((s) => alive && setState((prev) => prev ?? s));
    api.load().then((l) => alive && setCfg(l.config));
    return () => {
      alive = false;
      unlisten.forEach((u) => u.then((f) => f()));
    };
  }, []);

  return { state, level, cfg, t: T[cfg?.language ?? "vi"] };
}

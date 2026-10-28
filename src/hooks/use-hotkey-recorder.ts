import { useEffect, useRef, useState } from "react";
import { api, type HotkeyMode } from "@/lib/api";
import { keyCombo, mouseCombo, normalize, RING_SLOT } from "@/lib/hotkeys";

export function useHotkeyRecorder(onRecord: (slot: number, key: string) => void, hideOnEscape: boolean, mode: HotkeyMode) {
  const [recording, setRecording] = useState<number | null>(null);
  const first = useRef(true);
  const active = useRef(false);

  useEffect(() => {
    active.current = recording !== null;
    if (first.current) first.current = false;
    else api.setListening(recording !== null);
  }, [recording]);

  useEffect(
    () => () => {
      if (active.current) api.setListening(false);
    },
    [],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      if (recording !== null) {
        e.preventDefault();
        if (e.code === "Escape") return setRecording(null);
        if (/^(Control|Shift|Alt|Meta|OS)(Left|Right)?$/.test(e.code)) return;
        const key = normalize(e.code);
        setRecording(null);
        onRecord(recording, mode === "manual" ? keyCombo(e, key) : key);
      } else if (e.code === "Escape" && hideOnEscape) {
        api.hide();
      }
    };
    const onMouse = (e: MouseEvent) => {
      if (recording !== RING_SLOT || !mouseCombo(e)) return;
      e.preventDefault();
      if (e.type !== "mousedown") return;
      const combo = mouseCombo(e)!;
      setRecording(null);
      onRecord(recording, combo);
    };
    const mouseEvents = ["mousedown", "mouseup", "auxclick"] as const;
    window.addEventListener("keydown", onKey);
    mouseEvents.forEach((n) => window.addEventListener(n, onMouse));
    return () => {
      window.removeEventListener("keydown", onKey);
      mouseEvents.forEach((n) => window.removeEventListener(n, onMouse));
    };
  });

  return { recording, setRecording };
}

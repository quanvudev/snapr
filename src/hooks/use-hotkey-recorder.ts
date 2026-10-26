import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { normalize } from "@/lib/hotkeys";

export function useHotkeyRecorder(onRecord: (slot: number, key: string) => void, hideOnEscape: boolean) {
  const [recording, setRecording] = useState<number | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      if (recording !== null) {
        e.preventDefault();
        if (e.code === "Escape") return setRecording(null);
        if (/^(Control|Shift|Alt|Meta|OS)(Left|Right)?$/.test(e.code)) return;
        const key = normalize(e.code);
        setRecording(null);
        onRecord(recording, key);
      } else if (e.code === "Escape" && hideOnEscape) {
        api.hide();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  return { recording, setRecording };
}

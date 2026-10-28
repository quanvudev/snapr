import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { useSettingsContext } from "@/components/settings-provider";
import { normaliseKey } from "@/lib/ring";

const CLEAR_CODES = ["Backspace", "Delete"];

type Props = {
  configured: string | null | undefined;
  auto: string | null;
  onChange: (key: string | null) => void;
  onRecording: (recording: boolean) => void;
};

export function KeyRecorder({ configured, auto, onChange, onRecording }: Props) {
  const { t } = useSettingsContext();
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    onRecording(recording);
    return () => onRecording(false);
  }, [recording, onRecording]);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.code === "Escape") return setRecording(false);
      if (CLEAR_CODES.includes(e.code)) {
        onChange(null);
        return setRecording(false);
      }
      const key = normaliseKey(e.code);
      if (!key) return;
      onChange(key);
      setRecording(false);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, onChange]);

  return (
    <div className="flex items-center gap-2">
      {configured ? (
        <Kbd>{configured}</Kbd>
      ) : (
        <span className="flex items-center gap-1 text-sm text-muted-foreground">
          {t.ringKeyAuto}
          {auto && <Kbd>{auto}</Kbd>}
        </span>
      )}
      <Button variant="outline" size="sm" onClick={() => setRecording(!recording)}>
        {recording ? t.ringKeyPress : t.ringKeyRecord}
      </Button>
      {configured && !recording && (
        <Button variant="ghost" size="sm" onClick={() => onChange(null)}>
          {t.ringKeyClear}
        </Button>
      )}
    </div>
  );
}

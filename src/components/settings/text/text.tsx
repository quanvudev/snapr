import { useSettingsContext } from "@/components/settings-provider";
import { ModelStatusRow } from "../models/model-status-row";
import { usePaddleStore } from "../models/stores";
import { SettingsCard } from "../settings-card";
import { OcrEngineRow } from "./ocr-engine-row";
import { TextCaptureRow } from "./text-capture-row";

export function Text() {
  const { cfg } = useSettingsContext();
  const paddle = usePaddleStore();
  return (
    <SettingsCard>
      <TextCaptureRow />
      <OcrEngineRow />
      {cfg.ocr_engine === "paddle" && <ModelStatusRow store={paddle} />}
    </SettingsCard>
  );
}

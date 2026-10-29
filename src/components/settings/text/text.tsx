import { useSettingsContext } from "@/components/settings-provider";
import { ItemGroup } from "@/components/ui/item";
import { OcrEngineRow } from "./ocr-engine-row";
import { PaddleModelRow } from "./paddle-model-row";
import { PaddleModelsRow } from "./paddle-models-row";
import { TextCaptureRow } from "./text-capture-row";

export function Text() {
  const { cfg } = useSettingsContext();
  return (
    <ItemGroup className="gap-3">
      <TextCaptureRow />
      <OcrEngineRow />
      {cfg.ocr_engine === "paddle" && (
        <>
          <PaddleModelRow />
          <PaddleModelsRow />
        </>
      )}
    </ItemGroup>
  );
}

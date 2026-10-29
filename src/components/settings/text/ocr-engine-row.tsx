import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { OcrEngine } from "@/lib/api";

const ENGINES: OcrEngine[] = ["windows", "paddle"];

export function OcrEngineRow() {
  const { cfg, t, update } = useSettingsContext();
  const labels: Record<OcrEngine, string> = { windows: t.engineWindows, paddle: t.enginePaddle };
  const change = (value: string[]) => {
    const ocr_engine = value[0] as OcrEngine | undefined;
    if (ocr_engine) update({ ...cfg, ocr_engine });
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.ocrEngine}</ItemTitle>
        <ItemDescription>{t.ocrEngineHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[cfg.ocr_engine]} onValueChange={change}>
          {ENGINES.map((e) => (
            <ToggleGroupItem
              key={e}
              value={e}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {labels[e]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
    </Item>
  );
}

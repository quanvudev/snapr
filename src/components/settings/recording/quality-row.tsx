import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { Quality } from "@/lib/api";
import { QUALITIES } from "@/lib/queue";

export function QualityRow() {
  const { cfg, t, update } = useSettingsContext();
  const change = (value: string[]) => {
    if (value[0]) update({ ...cfg, record_quality: value[0] as Quality });
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.recordQuality}</ItemTitle>
        <ItemDescription>{t.recordQualityHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[cfg.record_quality]} onValueChange={change}>
          {QUALITIES.map((q) => (
            <ToggleGroupItem
              key={q.value}
              value={q.value}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {t[q.label]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
    </Item>
  );
}

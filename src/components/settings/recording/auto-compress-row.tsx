import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";

const HEIGHTS = [1080, 720];
const DEFAULT_HEIGHT = 1080;

export function AutoCompressRow() {
  const { cfg, t, update } = useSettingsContext();
  const changeHeight = (value: string[]) => {
    if (value[0]) update({ ...cfg, record_compress_height: Number(value[0]) });
  };
  const height = HEIGHTS.includes(cfg.record_compress_height) ? cfg.record_compress_height : DEFAULT_HEIGHT;

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.autoCompress}</ItemTitle>
        <ItemDescription>{t.autoCompressHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        {cfg.record_compress && (
          <ToggleGroup variant="outline" size="sm" value={[String(height)]} onValueChange={changeHeight}>
            {HEIGHTS.map((h) => (
              <ToggleGroupItem
                key={h}
                value={String(h)}
                className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
              >
                {h}p
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        )}
        <Switch
          checked={cfg.record_compress}
          onCheckedChange={(record_compress) => update({ ...cfg, record_compress })}
        />
      </ItemActions>
    </Item>
  );
}

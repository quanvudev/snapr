import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";

const FPS = [30, 60];

export function FpsRow() {
  const { cfg, t, update } = useSettingsContext();
  const change = (value: string[]) => {
    if (value[0]) update({ ...cfg, record_fps: Number(value[0]) });
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.fps}</ItemTitle>
        <ItemDescription>{t.fpsHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[String(cfg.record_fps === 60 ? 60 : 30)]} onValueChange={change}>
          {FPS.map((f) => (
            <ToggleGroupItem
              key={f}
              value={String(f)}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {f} fps
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
    </Item>
  );
}

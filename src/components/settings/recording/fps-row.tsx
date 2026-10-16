import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useSettingsContext } from "@/components/settings-provider";

const FPS = [30, 60];

export function FpsRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.fps}</ItemTitle>
        <ItemDescription>{t.fpsHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Select
          value={String(cfg.record_fps === 60 ? 60 : 30)}
          onValueChange={(v) => update({ ...cfg, record_fps: Number(v) })}
        >
          <SelectTrigger>
            <SelectValue>{(v: string) => `${v} fps`}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {FPS.map((f) => (
              <SelectItem key={f} value={String(f)}>
                {f} fps
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </ItemActions>
    </Item>
  );
}

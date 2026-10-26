import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useSettingsContext } from "@/components/settings-provider";
import type { Theme } from "@/lib/api";

const THEMES: Theme[] = ["system", "light", "dark"];

export function ThemeRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.theme}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Select value={cfg.theme} onValueChange={(v) => update({ ...cfg, theme: v as Theme })}>
          <SelectTrigger>
            <SelectValue>{(v: Theme) => t[v]}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {THEMES.map((k) => (
              <SelectItem key={k} value={k}>
                {t[k]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </ItemActions>
    </Item>
  );
}

import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useSettingsContext } from "@/components/settings-provider";
import { LANGS } from "@/lib/i18n";

export function LanguageRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.language}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Select value={cfg.language} onValueChange={(v) => update({ ...cfg, language: v as keyof typeof LANGS })}>
          <SelectTrigger>
            <SelectValue>{(v: keyof typeof LANGS) => LANGS[v]}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {Object.entries(LANGS).map(([k, label]) => (
              <SelectItem key={k} value={k}>
                {label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </ItemActions>
    </Item>
  );
}

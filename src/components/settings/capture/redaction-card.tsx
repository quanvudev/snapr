import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { Redact, RedactStyle } from "@/lib/api";
import type { Strings } from "@/lib/i18n";
import { SettingsCard } from "../settings-card";
import { RedactEngineRow } from "./redact-engine-row";
import { RedactPatterns } from "./redact-patterns";

type Category = "email" | "phone" | "card" | "ipv4" | "secrets";

const CATEGORIES: { key: Category; label: keyof Strings }[] = [
  { key: "email", label: "redactEmail" },
  { key: "phone", label: "redactPhone" },
  { key: "card", label: "redactCard" },
  { key: "ipv4", label: "redactIp" },
  { key: "secrets", label: "redactSecrets" },
];

const STYLES: { value: RedactStyle; label: keyof Strings }[] = [
  { value: "black", label: "redactBlack" },
  { value: "pixelate", label: "redactPixelate" },
];

const PRESSED = "text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground";

export function RedactionCard() {
  const { cfg, t, ocr, update, setSection } = useSettingsContext();
  const set = (patch: Partial<Redact>) => update({ ...cfg, redact: { ...cfg.redact, ...patch } });
  return (
    <SettingsCard>
      {!ocr && (
        <Item variant="muted" size="sm">
          <ItemContent>
            <ItemDescription>{t.redactNoOcr}</ItemDescription>
          </ItemContent>
          <ItemActions>
            <Button variant="link" size="sm" onClick={() => setSection("text")}>
              {`${t.text} →`}
            </Button>
          </ItemActions>
        </Item>
      )}
      <Item variant="muted">
        <ItemContent>
          <ItemTitle>{t.redactAuto}</ItemTitle>
          <ItemDescription>{t.redactAutoHint}</ItemDescription>
        </ItemContent>
        <ItemActions>
          <Switch checked={cfg.redact.auto_capture} onCheckedChange={(auto_capture) => set({ auto_capture })} />
        </ItemActions>
      </Item>
      <Item variant="muted">
        <ItemContent>
          <ItemTitle>{t.redactStyle}</ItemTitle>
        </ItemContent>
        <ItemActions>
          <ToggleGroup
            variant="outline"
            size="sm"
            value={[cfg.redact.style]}
            onValueChange={(v) => v[0] && set({ style: v[0] as RedactStyle })}
          >
            {STYLES.map((s) => (
              <ToggleGroupItem key={s.value} value={s.value} className={PRESSED}>
                {t[s.label]}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </ItemActions>
      </Item>
      <RedactEngineRow />
      {CATEGORIES.map((c) => (
        <Item key={c.key} variant="muted" size="sm">
          <ItemContent>
            <ItemTitle>{t[c.label]}</ItemTitle>
          </ItemContent>
          <ItemActions>
            <Switch checked={cfg.redact[c.key]} onCheckedChange={(on) => set({ [c.key]: on })} />
          </ItemActions>
        </Item>
      ))}
      <RedactPatterns patterns={cfg.redact.patterns} onChange={(patterns) => set({ patterns })} />
    </SettingsCard>
  );
}

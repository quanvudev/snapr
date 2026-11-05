import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { Redact } from "@/lib/api";
import { paddleLabels } from "../models/paddle-labels";

const PRESSED = "text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground";

export function RedactEngineRow() {
  const { cfg, t, paddle, update, setSection } = useSettingsContext();
  const ready = paddle.status.ready.includes(cfg.paddle_model);
  const size = paddleLabels(t).sizes[cfg.paddle_model];
  const engine = cfg.redact.engine === "paddle" && ready ? "paddle" : "windows";
  const hint = !ready
    ? t.redactEngineMissing.replace("{size}", size)
    : engine === "paddle"
      ? t.redactEnginePaddleHint.replace("{size}", size)
      : t.redactEngineWindowsHint;
  return (
    <Item variant="muted" className="flex-wrap">
      <ItemContent>
        <ItemTitle>{t.redactEngine}</ItemTitle>
        <ItemDescription>{hint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[engine]}
          onValueChange={(v) => v[0] && update({ ...cfg, redact: { ...cfg.redact, engine: v[0] as Redact["engine"] } })}
        >
          <ToggleGroupItem value="windows" className={PRESSED}>
            {t.redactEngineWindows}
          </ToggleGroupItem>
          <ToggleGroupItem value="paddle" disabled={!ready} className={PRESSED}>
            {t.redactEnginePaddle}
          </ToggleGroupItem>
        </ToggleGroup>
      </ItemActions>
      {!ready && (
        <ItemFooter>
          <Button variant="link" size="sm" className="px-0" onClick={() => setSection("models")}>
            {`${t.models} →`}
          </Button>
        </ItemFooter>
      )}
    </Item>
  );
}

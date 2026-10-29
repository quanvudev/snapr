import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Progress, ProgressValue } from "@/components/ui/progress";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { PaddleSize } from "@/lib/api";
import { paddleLabels } from "./paddle-labels";

const SIZES: PaddleSize[] = ["tiny", "small", "medium"];

export function PaddleModelRow() {
  const { cfg, t, paddle, update } = useSettingsContext();
  const { sizes, descs } = paddleLabels(t);
  const size = cfg.paddle_model;
  const active = paddle.progress;
  const failed = paddle.error?.size === size ? paddle.error.message : null;
  const ready = paddle.status.ready.includes(size);
  const change = (value: string[]) => {
    const paddle_model = value[0] as PaddleSize | undefined;
    if (paddle_model) update({ ...cfg, paddle_model });
  };
  const percent = active && active.total > 0 ? (active.done / active.total) * 100 : 0;

  return (
    <Item variant="muted" className="flex-wrap">
      <ItemContent>
        <ItemTitle>{t.paddleModel}</ItemTitle>
        <ItemDescription>{descs[size]}</ItemDescription>
        <ItemDescription>{t.paddleNoVi}</ItemDescription>
        <ItemDescription>{t.runtimeNote}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[size]} onValueChange={change}>
          {SIZES.map((s) => (
            <ToggleGroupItem
              key={s}
              value={s}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {sizes[s]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
      {active ? (
        <div className="flex basis-full items-center gap-3">
          <Progress value={percent} className="flex-1">
            <span className="text-sm">{t.paddleDownloading.replace("{size}", sizes[active.size])}</span>
            <ProgressValue>{() => `${Math.round(percent)}%`}</ProgressValue>
          </Progress>
          <Button size="sm" variant="outline" onClick={paddle.cancel}>
            {t.cancel}
          </Button>
        </div>
      ) : failed ? (
        <div className="flex basis-full items-center justify-between gap-3">
          <span className="text-sm text-destructive">{t.paddleFailed.replace("{error}", failed)}</span>
          <Button size="sm" variant="outline" onClick={() => paddle.download(size)}>
            {t.paddleRetry}
          </Button>
        </div>
      ) : (
        !ready && (
          <div className="flex basis-full items-center justify-between gap-3">
            <span className="text-sm text-muted-foreground">{t.paddleNotDownloaded}</span>
            <Button size="sm" variant="outline" onClick={() => paddle.download(size)}>
              {t.paddleDownload}
            </Button>
          </div>
        )
      )}
    </Item>
  );
}

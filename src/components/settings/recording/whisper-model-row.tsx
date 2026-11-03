import { useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemGroup, ItemTitle } from "@/components/ui/item";
import { Progress, ProgressValue } from "@/components/ui/progress";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { WhisperModel } from "@/lib/api";
import { mb, WHISPER_MODELS } from "./whisper-models";

export function WhisperModelRow() {
  const { cfg, t, whisper, update } = useSettingsContext();
  const [target, setTarget] = useState<WhisperModel | null>(null);
  const model = cfg.whisper_model;
  const active = whisper.progress;
  const failed = whisper.error?.model === model ? whisper.error.message : null;
  const ready = whisper.status.ready.includes(model);
  const label = (m: WhisperModel) => t[WHISPER_MODELS.find((w) => w.value === m)!.label];
  const change = (value: string[]) => {
    const whisper_model = value[0] as WhisperModel | undefined;
    if (whisper_model) update({ ...cfg, whisper_model });
  };
  const percent = active && active.total > 0 ? (active.done / active.total) * 100 : 0;

  return (
    <>
      <Item variant="muted" className="flex-wrap">
        <ItemContent>
          <ItemTitle>{t.whisperModel}</ItemTitle>
        </ItemContent>
        <ItemActions>
          <ToggleGroup variant="outline" size="sm" value={[model]} onValueChange={change}>
            {WHISPER_MODELS.map((m) => (
              <ToggleGroupItem
                key={m.value}
                value={m.value}
                className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
              >
                {t[m.label]}
                {whisper.status.bytes[m.value] > 0 && ` · ${mb(whisper.status.bytes[m.value])}`}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </ItemActions>
        {active ? (
          <div className="flex basis-full items-center gap-3">
            <Progress value={percent} className="flex-1">
              <span className="text-sm">{t.whisperDownloading.replace("{model}", label(active.model))}</span>
              <ProgressValue>{() => `${Math.round(percent)}%`}</ProgressValue>
            </Progress>
            <Button size="sm" variant="outline" onClick={whisper.cancel}>
              {t.cancel}
            </Button>
          </div>
        ) : failed ? (
          <div className="flex basis-full items-center justify-between gap-3">
            <span className="text-sm text-destructive">{t.whisperFailed.replace("{error}", failed)}</span>
            <Button size="sm" variant="outline" onClick={() => whisper.download(model)}>
              {t.paddleRetry}
            </Button>
          </div>
        ) : (
          !ready && (
            <div className="flex basis-full items-center justify-between gap-3">
              <span className="text-sm text-muted-foreground">{t.paddleNotDownloaded}</span>
              <Button size="sm" variant="outline" onClick={() => whisper.download(model)}>
                {t.paddleDownload}
              </Button>
            </div>
          )
        )}
      </Item>
      {whisper.status.ready.length > 0 && (
        <ItemGroup className="gap-1">
          <span className="px-1 text-sm font-medium">{t.whisperModels}</span>
          {WHISPER_MODELS.filter((m) => whisper.status.ready.includes(m.value)).map((m) => (
            <Item key={m.value} variant="muted" size="sm">
              <ItemContent>
                <ItemTitle>
                  {t[m.label]}
                  <Badge variant={cfg.whisper_model === m.value ? "default" : "secondary"}>
                    {cfg.whisper_model === m.value ? t.paddleInUse : t.paddleDownloaded}
                  </Badge>
                </ItemTitle>
              </ItemContent>
              <ItemActions className="gap-3">
                <span className="text-sm text-muted-foreground">{mb(whisper.status.bytes[m.value])}</span>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={active?.model === m.value}
                  onClick={() => setTarget(m.value)}
                >
                  {t.paddleDelete}
                </Button>
              </ItemActions>
            </Item>
          ))}
        </ItemGroup>
      )}
      <AlertDialog open={target !== null} onOpenChange={(open) => !open && setTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{target && t.whisperDeleteTitle.replace("{model}", label(target))}</AlertDialogTitle>
            <AlertDialogDescription>
              {target && t.paddleDeleteHint.replace("{bytes}", mb(whisper.status.bytes[target]))}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (target) whisper.remove(target);
                setTarget(null);
              }}
            >
              {t.paddleDelete}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

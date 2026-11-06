import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Progress, ProgressValue } from "@/components/ui/progress";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { Acceleration, GpuRuntime, Runtime } from "@/lib/api";
import { PERCENT } from "@/lib/queue";
import { SettingsCard } from "../settings-card";
import { mb } from "./whisper-models";

const BYTES_PER_GB = 1073741824;
const NVIDIA = 0x10de;

export const RUNTIME_NAMES: Record<Runtime, string> = { cpu: "CPU", directml: "DirectML", cuda: "CUDA" };

export function useAccelerationErrors() {
  const { t } = useSettingsContext();
  const known: Record<string, string> = {
    RECORDING: t.accelErrorRecording,
    QUEUE_BUSY: t.accelErrorQueue,
    LOADED_RUNTIME: t.accelErrorLoaded,
    DOWNLOADING: t.accelErrorDownloading,
  };
  return (error: string) => known[error] ?? error;
}

export function AccelerationCard() {
  const { cfg, t, update, acceleration } = useSettingsContext();
  const { status, progress, error, restartError } = acceleration;
  const message = useAccelerationErrors();
  const adapter = status.adapter;
  const nvidia = adapter?.vendor === NVIDIA;
  const selected = cfg.ai_acceleration;
  const choices: Acceleration[] = ["auto", "cpu", "directml", ...(nvidia ? (["cuda"] as const) : [])];
  const label = (choice: Acceleration) => (choice === "auto" ? t.accelAuto : RUNTIME_NAMES[choice]);
  const runtime = selected === "directml" || selected === "cuda" ? status.runtimes.find((r) => r.id === selected) : null;
  const failed = error && error.runtime === selected ? error.message : null;
  const percent = progress && progress.total > 0 ? (progress.done / progress.total) * PERCENT : 0;
  const inUse = status.loaded ?? status.resolved;
  const notes: Record<Acceleration, string> = {
    auto: t.accelAutoNote,
    cpu: t.accelCpuNote,
    directml: t.accelDirectmlNote,
    cuda: t.accelCudaNote,
  };
  const card = adapter ? `${adapter.name} · ${Math.round(adapter.memory / BYTES_PER_GB)} GB` : t.accelNoCard;
  const recommend =
    status.recommended !== "cpu" && status.recommended !== inUse
      ? ` · ${t.accelRecommend.replace("{runtime}", RUNTIME_NAMES[status.recommended])}`
      : "";
  const change = (value: string[]) => {
    if (value[0]) update({ ...cfg, ai_acceleration: value[0] as Acceleration });
  };

  return (
    <SettingsCard>
      <Item variant="muted" className="flex-wrap">
        <ItemContent>
          <ItemTitle>{card}</ItemTitle>
          <ItemDescription>
            {t.accelInUse.replace("{runtime}", RUNTIME_NAMES[inUse])}
            {recommend}
          </ItemDescription>
        </ItemContent>
        <ItemActions>
          <ToggleGroup variant="outline" size="sm" value={[selected]} onValueChange={change}>
            {choices.map((choice) => (
              <ToggleGroupItem
                key={choice}
                value={choice}
                disabled={!adapter && (choice === "directml" || choice === "cuda")}
                className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
              >
                {label(choice)}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </ItemActions>
        <p className="basis-full text-sm text-muted-foreground">{notes[selected]}</p>
        {progress ? (
          <div className="flex basis-full items-center gap-3">
            <Progress value={percent} className="flex-1">
              <span className="text-sm">
                {t.modelDownloading.replace("{name}", t.accelRuntime.replace("{name}", RUNTIME_NAMES[progress.runtime]))}
              </span>
              <ProgressValue>{() => `${Math.round(percent)}%`}</ProgressValue>
            </Progress>
            <Button size="sm" variant="outline" onClick={acceleration.cancel}>
              {t.cancel}
            </Button>
          </div>
        ) : failed ? (
          <div className="flex basis-full items-center justify-between gap-3">
            <span className="text-sm text-destructive">{t.paddleFailed.replace("{error}", message(failed))}</span>
            <Button size="sm" variant="outline" onClick={() => acceleration.download(selected as GpuRuntime)}>
              {t.paddleRetry}
            </Button>
          </div>
        ) : (
          runtime &&
          !runtime.ready && (
            <div className="flex basis-full items-center justify-between gap-3">
              <span className="text-sm text-muted-foreground">
                {t.accelNotDownloaded.replace("{name}", RUNTIME_NAMES[runtime.id]).replace("{size}", mb(runtime.bytes))}
              </span>
              <Button size="sm" variant="outline" onClick={() => acceleration.download(runtime.id)}>
                {t.paddleDownload}
              </Button>
            </div>
          )
        )}
        {status.restart_needed && (
          <div className="flex basis-full items-center justify-between gap-3">
            <span className="text-sm">
              {t.accelRestartNeeded}
              {restartError && <span className="block text-destructive">{message(restartError)}</span>}
            </span>
            <Button size="sm" onClick={acceleration.restart}>
              {t.accelRestart}
            </Button>
          </div>
        )}
        {status.error && (
          <p className="basis-full text-sm text-destructive">{t.accelGpuError.replace("{error}", status.error)}</p>
        )}
      </Item>
    </SettingsCard>
  );
}

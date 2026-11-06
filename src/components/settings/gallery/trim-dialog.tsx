import { useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Slider } from "@/components/ui/slider";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type GalleryItem } from "@/lib/api";
import { fileName } from "@/lib/gallery";
import { GIF_SLIDER_STEP_MS, MS_PER_SECOND } from "@/lib/queue";

type Props = { item: GalleryItem | null; onClose: () => void; onTrim: (item: GalleryItem, startMs: number, endMs: number) => void };

const seconds = (ms: number) => `${(ms / MS_PER_SECOND).toFixed(1)}s`;

export function TrimDialog({ item, onClose, onTrim }: Props) {
  const { t } = useSettingsContext();
  return (
    <Dialog open={!!item} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t.trimVideo}</DialogTitle>
        </DialogHeader>
        {item && <TrimForm key={item.path} item={item} onClose={onClose} onTrim={onTrim} />}
      </DialogContent>
    </Dialog>
  );
}

function TrimForm({ item, onClose, onTrim }: { item: GalleryItem; onClose: () => void; onTrim: Props["onTrim"] }) {
  const { t } = useSettingsContext();
  const video = useRef<HTMLVideoElement>(null);
  const [durationMs, setDurationMs] = useState<number | null>(null);
  const [range, setRange] = useState<[number, number] | null>(null);

  const [start, end] = range ?? [0, durationMs ?? 0];
  const length = end - start;
  const whole = start === 0 && end === durationMs;

  const changeRange = (value: number | readonly number[]) => {
    if (!Array.isArray(value)) return;
    const next: [number, number] = [value[0], value[1]];
    const moved = next[0] !== start ? next[0] : next[1];
    setRange(next);
    if (video.current) video.current.currentTime = moved / MS_PER_SECOND;
  };

  return (
    <>
      <p className="truncate text-sm text-muted-foreground">{fileName(item.path)}</p>
      <video
        ref={video}
        src={api.mediaSrc(item.path)}
        title={fileName(item.path)}
        controls
        className="max-h-72 w-full rounded-lg bg-black object-contain"
        onLoadedMetadata={(e) => setDurationMs(Math.floor(e.currentTarget.duration * MS_PER_SECOND))}
      />
      <div className="grid gap-2">
        <Slider
          aria-label={t.gifRange}
          disabled={durationMs === null}
          min={0}
          max={durationMs ?? 1}
          step={GIF_SLIDER_STEP_MS}
          minStepsBetweenValues={1}
          value={[start, end]}
          onValueChange={changeRange}
        />
        <div className="flex justify-between text-xs text-muted-foreground tabular-nums">
          <span>
            {t.gifStart}: {seconds(start)}
          </span>
          <span>
            {t.gifDuration}: {seconds(length)}
          </span>
          <span>
            {t.gifEnd}: {seconds(end)}
          </span>
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t.cancel}
        </Button>
        <Button disabled={durationMs === null || length <= 0 || whole} onClick={() => onTrim(item, start, end)}>
          {t.trimAction}
        </Button>
      </DialogFooter>
    </>
  );
}

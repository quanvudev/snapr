import { useEffect, useRef, useState } from "react";
import { Check, Copy, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { useSettingsContext } from "@/components/settings-provider";
import { useTranscript } from "@/hooks/use-transcript";
import { formatDuration } from "@/lib/gallery";
import { MS_PER_SECOND } from "@/lib/queue";
import { cn } from "@/lib/utils";

const FOLLOW_PAUSE_MS = 3000;
const COPIED_MS = 1500;

type Props = { path: string; video: HTMLVideoElement | null; onClose: () => void };

export function TranscriptPane({ path, video, onClose }: Props) {
  const { t } = useSettingsContext();
  const { segments, active, running, error, create, cancel, seek } = useTranscript(path, video);
  const [copied, setCopied] = useState(false);
  const list = useRef<HTMLDivElement>(null);
  const lastUserScroll = useRef(0);
  const all = segments?.map((s) => s.text).join("\n") ?? "";

  useEffect(() => {
    if (active < 0 || Date.now() - lastUserScroll.current < FOLLOW_PAUSE_MS) return;
    list.current?.querySelector(`[data-index="${active}"]`)?.scrollIntoView({ block: "nearest" });
  }, [active]);

  const markScroll = () => {
    lastUserScroll.current = Date.now();
  };

  const copy = () =>
    navigator.clipboard.writeText(all).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), COPIED_MS);
    });

  return (
    <aside className="flex w-80 shrink-0 flex-col rounded-lg border bg-muted/40">
      <div className="flex items-center gap-1 border-b p-2">
        <span className="flex-1 pl-1 text-sm font-medium">{t.transcript}</span>
        <Button variant="ghost" size="icon-sm" title={t.copyAll} disabled={!all} onClick={copy}>
          {copied ? <Check /> : <Copy />}
        </Button>
        <Button variant="ghost" size="icon-sm" title={t.close} onClick={onClose}>
          <X />
        </Button>
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <div
          ref={list}
          className="space-y-1 p-2 text-sm"
          onWheel={markScroll}
          onPointerDown={markScroll}
          onTouchMove={markScroll}
        >
          {running ? (
            <div className="space-y-2 p-2">
              <p className="text-muted-foreground">
                {running.waiting
                  ? t.transcriptWaiting
                  : t.transcriptRunning.replace("{percent}", String(Math.round(running.percent)))}
              </p>
              <Progress value={running.percent} />
              <Button size="sm" variant="outline" onClick={cancel}>
                {t.cancel}
              </Button>
            </div>
          ) : segments === undefined ? (
            <div className="flex justify-center py-8">
              <Spinner />
            </div>
          ) : error ? (
            <div className="space-y-2 p-2">
              <p className="text-destructive">{t.transcriptFailed.replace("{error}", error)}</p>
              <Button size="sm" variant="outline" onClick={create}>
                {t.paddleRetry}
              </Button>
            </div>
          ) : segments === null ? (
            <div className="flex justify-center p-2">
              <Button size="sm" variant="outline" onClick={create}>
                {t.transcriptCreate}
              </Button>
            </div>
          ) : segments.length === 0 ? (
            <p className="p-2 text-muted-foreground">{t.transcriptNone}</p>
          ) : (
            segments.map((segment, i) => (
              <Button
                key={i}
                data-index={i}
                variant="ghost"
                className={cn("h-auto w-full items-start justify-start gap-2 px-2 py-1.5 text-left whitespace-normal", i === active && "bg-accent")}
                onClick={() => seek(segment.start_ms)}
              >
                <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
                  {formatDuration(segment.start_ms / MS_PER_SECOND)}
                </span>
                <span className="min-w-0 flex-1 break-words">{segment.text}</span>
              </Button>
            ))
          )}
        </div>
      </ScrollArea>
    </aside>
  );
}

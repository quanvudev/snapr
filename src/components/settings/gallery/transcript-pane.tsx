import { useEffect, useRef, useState } from "react";
import { Captions, Check, Copy, Languages, Pencil, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useSettingsContext } from "@/components/settings-provider";
import { activeSegment, useTranscript, useTranslationJobs } from "@/hooks/use-transcript";
import type { GalleryItem, Track } from "@/lib/api";
import { canTranslate, formatDuration, itemTracks } from "@/lib/gallery";
import { languageName, trackName } from "@/lib/languages";
import { MS_PER_SECOND } from "@/lib/queue";
import { cn } from "@/lib/utils";

const FOLLOW_PAUSE_MS = 3000;
const COPIED_MS = 1500;

type Props = {
  item: GalleryItem;
  track: Track;
  onTrack: (track: Track) => void;
  onTranslate: () => void;
  video: HTMLVideoElement | null;
  onClose: () => void;
  onExport: (ms: number, text: string) => void;
};

export function TranscriptPane({ item, track: chosen, onTrack, onTranslate, video, onClose, onExport }: Props) {
  const { cfg, t, translator } = useSettingsContext();
  const hymt = translator.status.ready;
  const jobs = useTranslationJobs(item.path);
  const [seen, setSeen] = useState<{ path: string; targets: string[] }>({ path: item.path, targets: [] });
  const recent = seen.path === item.path ? seen.targets : [];
  useEffect(() => {
    if (jobs.length === 0) return;
    setSeen((prev) => {
      const base = prev.path === item.path ? prev.targets : [];
      const next = [...new Set([...base, ...jobs])];
      return next.length === base.length && prev.path === item.path ? prev : { path: item.path, targets: next };
    });
  }, [jobs, item.path]);
  const tracks = [...new Set<Track>([...itemTracks(item, hymt), ...recent, ...jobs])];
  const track = tracks.includes(chosen) ? chosen : "original";
  const translatable = hymt && canTranslate(item, true);
  const { segments, active, running, error, create, save, cancel, seek } = useTranscript(item, track, video);
  const [copied, setCopied] = useState(false);
  const [editing, setEditing] = useState<number | null>(null);
  const list = useRef<HTMLDivElement>(null);
  const lastUserScroll = useRef(0);
  const all = segments?.map((s) => s.text).join("\n") ?? "";
  const hasText = !!segments?.some((s) => s.text.trim());

  const exportSubtitles = () => {
    if (!segments) return;
    const current = segments[activeSegment(segments, (video?.currentTime ?? 0) * MS_PER_SECOND)];
    const cue = current?.text.trim() ? current : segments.find((s) => s.text.trim());
    onExport(video ? video.currentTime * MS_PER_SECOND : 0, cue?.text.trim() ?? "");
  };

  const commit = (index: number, text: string) => {
    setEditing(null);
    if (text !== segments?.[index].text) void save(index, text);
  };

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
        {translatable && (
          <Button variant="ghost" size="icon-sm" title={t.translateSubtitles} onClick={onTranslate}>
            <Languages />
          </Button>
        )}
        <Button variant="ghost" size="icon-sm" title={t.copyAll} disabled={!all} onClick={copy}>
          {copied ? <Check /> : <Copy />}
        </Button>
        <Button variant="ghost" size="icon-sm" title={t.exportSubtitles} disabled={!hasText || !!running} onClick={exportSubtitles}>
          <Captions />
        </Button>
        <Button variant="ghost" size="icon-sm" title={t.close} onClick={onClose}>
          <X />
        </Button>
      </div>
      {tracks.length > 1 && (
        <div className="border-b p-2">
          <Select value={track} onValueChange={(v) => v && onTrack(v as Track)}>
            <SelectTrigger size="sm" className="w-full">
              <SelectValue>{(v: Track) => trackName(v, cfg.language, t.trackOriginal, item.language)}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              {tracks.map((tr) => (
                <SelectItem key={tr} value={tr}>
                  {trackName(tr, cfg.language, t.trackOriginal, item.language)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}
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
                {track === "original"
                  ? t.transcriptCreate
                  : hymt
                    ? t.translateIntoLanguage.replace("{language}", languageName(track, cfg.language))
                    : t.translateToEnglish}
              </Button>
            </div>
          ) : segments.length === 0 ? (
            <p className="p-2 text-muted-foreground">{t.transcriptNone}</p>
          ) : (
            segments.map((segment, i) =>
              editing === i ? (
                <Input
                  key={i}
                  data-index={i}
                  autoFocus
                  defaultValue={segment.text}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") commit(i, e.currentTarget.value);
                    else if (e.key === "Escape") setEditing(null);
                  }}
                  onBlur={() => setEditing(null)}
                />
              ) : (
                <div key={i} data-index={i} className="group relative">
                  <Button
                    variant="ghost"
                    className={cn("h-auto w-full items-start justify-start gap-2 px-2 py-1.5 pr-9 text-left whitespace-normal", i === active && "bg-accent")}
                    onClick={() => seek(segment.start_ms)}
                    onDoubleClick={() => !running && setEditing(i)}
                  >
                    <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
                      {formatDuration(segment.start_ms / MS_PER_SECOND)}
                    </span>
                    <span className="min-w-0 flex-1 break-words">{segment.text}</span>
                  </Button>
                  {!running && (
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      title={t.editSegment}
                      className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
                      onClick={() => setEditing(i)}
                    >
                      <Pencil />
                    </Button>
                  )}
                </div>
              ),
            )
          )}
        </div>
      </ScrollArea>
    </aside>
  );
}

import { useEffect, useRef, useState } from "react";
import { Captions, Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useTheme } from "@/hooks/use-theme";
import {
  api,
  type CaptionSource,
  type Config,
  type LiveCaption,
} from "@/lib/api";
import { T } from "@/lib/i18n";
import "@/App.css";

type Line = {
  id: number;
  text: string;
  final: boolean;
  source: CaptionSource;
  translation?: string;
};

const FOLLOW_SLACK = 24;
const COPIED_MS = 1500;

function merge(lines: Line[], caption: LiveCaption): Line[] {
  const at = lines.findIndex((l) => l.id === caption.id);
  if ("translation" in caption) {
    if (at < 0) return lines;
    return lines.map((l, i) =>
      i === at ? { ...l, translation: caption.translation } : l,
    );
  }
  if (caption.final && !caption.text)
    return lines.filter((l) => l.id !== caption.id);
  const next = {
    id: caption.id,
    text: caption.text,
    final: caption.final,
    source: caption.source,
    translation: at < 0 ? undefined : lines[at].translation,
  };
  return at < 0 ? [...lines, next] : lines.map((l, i) => (i === at ? next : l));
}

export function CaptionsApp() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [lines, setLines] = useState<Line[]>([]);
  const [copied, setCopied] = useState(false);
  const content = useRef<HTMLDivElement>(null);
  const follow = useRef(true);
  const timer = useRef<number | undefined>(undefined);
  useTheme(cfg?.theme);
  const t = T[cfg?.language ?? "vi"];

  useEffect(() => {
    for (const el of [
      document.documentElement,
      document.body,
      document.getElementById("root"),
    ]) {
      if (!el) continue;
      el.style.background = "transparent";
      el.style.overflow = "hidden";
    }
    let alive = true;
    const unlisten = api.onLiveCaption((caption) =>
      setLines((prev) => merge(prev, caption)),
    );
    api.load().then((l) => alive && setCfg(l.config));
    return () => {
      alive = false;
      window.clearTimeout(timer.current);
      unlisten.then((f) => f());
    };
  }, []);

  const viewport = () =>
    content.current?.closest<HTMLElement>("[data-slot=scroll-area-viewport]");

  useEffect(() => {
    const el = viewport();
    if (el && follow.current) el.scrollTop = el.scrollHeight;
  }, [lines]);

  const onScroll = () => {
    const el = viewport();
    if (el)
      follow.current =
        el.scrollHeight - el.scrollTop - el.clientHeight < FOLLOW_SLACK;
  };

  const copy = () => {
    const text = lines
      .filter((l) => l.final)
      .flatMap((l) => {
        const who = l.source === "mic" ? t.capYou : t.capSpeaker;
        return l.translation
          ? [`${who}: ${l.text}`, l.translation]
          : [`${who}: ${l.text}`];
      })
      .join("\n");
    api.captionsCopy(text).then(() => {
      setCopied(true);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => setCopied(false), COPIED_MS);
    });
  };

  return (
    <div className="flex h-screen flex-col overflow-hidden rounded-2xl border border-foreground/10 bg-popover/90 text-popover-foreground backdrop-blur">
      <div
        data-tauri-drag-region
        className="flex h-9 shrink-0 items-center gap-2 pr-1 pl-3 text-xs text-muted-foreground"
      >
        <Captions data-tauri-drag-region className="size-4" />
        <span data-tauri-drag-region className="flex-1 truncate">
          {t.recModeCaptions}
        </span>
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                className="rounded-full"
                aria-label={copied ? t.capCopied : t.capCopyAll}
                disabled={!lines.some((l) => l.final)}
                onClick={copy}
              />
            }
          >
            {copied ? <Check /> : <Copy />}
          </TooltipTrigger>
          <TooltipContent side="bottom">
            {copied ? t.capCopied : t.capCopyAll}
          </TooltipContent>
        </Tooltip>
      </div>
      <ScrollArea className="min-h-0 flex-1" onScrollCapture={onScroll}>
        <div ref={content} className="flex flex-col gap-2 px-4 pb-3">
          {lines.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t.capListening}</p>
          ) : null}
          {lines.map((line, i) => {
            const mine = line.source === "mic";
            const named = i === 0 || lines[i - 1].source !== line.source;
            return (
              <div
                key={line.id}
                className={`flex max-w-[85%] flex-col gap-0.5 ${mine ? "items-end self-end" : "items-start self-start"}`}
              >
                {named ? (
                  <span className="px-1 text-[11px] text-muted-foreground">
                    {mine ? t.capYou : t.capSpeaker}
                  </span>
                ) : null}
                <div
                  className={`flex flex-col gap-0.5 rounded-2xl px-3 py-1.5 ${mine ? "rounded-br-sm bg-primary text-primary-foreground" : "rounded-bl-sm bg-muted text-foreground"}`}
                >
                  <p
                    className={`text-sm ${line.final ? "opacity-80" : "italic opacity-60"}`}
                  >
                    {line.text}
                  </p>
                  {line.translation ? (
                    <p className="text-base font-semibold">
                      {line.translation}
                    </p>
                  ) : null}
                </div>
              </div>
            );
          })}
        </div>
      </ScrollArea>
    </div>
  );
}

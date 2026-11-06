import { CornerDownLeft } from "lucide-react";
import { useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { LANGS } from "@/lib/i18n";
import icon from "../../../src-tauri/icons/128x128@2x.png";
import { ModifierKeys } from "./modifier-keys";
import { SETUP_SLOTS, ShortcutGrid } from "./shortcut-grid";

const caption = "px-0.5 text-[11px] font-medium tracking-wider text-muted-foreground uppercase";

export function Setup() {
  const { cfg, t, rows, failed, start, autostart, update } = useSettingsContext();
  const blocked = SETUP_SLOTS.some((i) => rows[i]?.error || rows[i]?.in_use);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const html = document.documentElement;
    const dark = html.classList.contains("dark");
    html.style.background = "transparent";
    html.classList.add("dark");
    return () => {
      html.style.background = "";
      html.classList.toggle("dark", dark);
    };
  }, []);

  useEffect(() => {
    const el = root.current;
    if (!el) return;
    const fit = () => api.setupFit(Math.ceil(el.getBoundingClientRect().height));
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    fit();
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Enter" || e.defaultPrevented || blocked) return;
      if (e.target instanceof Element && e.target.closest("button, [role=combobox], [role=listbox]")) return;
      start();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [blocked, start]);

  return (
    <div ref={root} className="bg-neutral-950/80 text-sm text-foreground">
      <div data-tauri-drag-region className="space-y-6 px-7 pt-8 pb-6">
        <div data-tauri-drag-region className="flex flex-col items-center gap-3 text-center">
          <img src={icon} alt="" className="pointer-events-none size-14 drop-shadow-[0_6px_18px_rgba(0,0,0,0.5)]" />
          <div className="pointer-events-none space-y-1">
            <h1 className="text-xl font-semibold">{t.welcome}</h1>
            <p className="text-muted-foreground">{t.welcomeSub}</p>
          </div>
        </div>

        <div className="space-y-2">
          <div className={caption}>{t.modifiers}</div>
          <ModifierKeys />
        </div>

        <div className="space-y-2">
          <div className={caption}>{t.thenPress}</div>
          <ShortcutGrid />
          <p className="px-0.5 text-xs text-muted-foreground">{t.moreShortcuts}</p>
        </div>

        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">{t.language}</span>
            <Select value={cfg.language} onValueChange={(v) => update({ ...cfg, language: v as keyof typeof LANGS })}>
              <SelectTrigger size="sm">
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
          </div>
          <div className="flex items-center gap-2">
            <Label htmlFor="setup-startup" className="font-normal text-muted-foreground">
              {t.startup}
            </Label>
            <Switch id="setup-startup" checked={autostart} onCheckedChange={(v) => update(cfg, v)} />
          </div>
        </div>
      </div>

      <div className="flex items-center gap-3 border-t border-white/10 bg-black/20 px-4 py-3">
        <Button variant="ghost" onClick={api.hide}>
          {t.quitSetup}
        </Button>
        <div className="ml-auto flex items-center gap-3">
          {failed && <p className="text-xs text-destructive">{t.failed}</p>}
          <Button disabled={blocked} onClick={start}>
            {t.start}
            <Kbd className="bg-black/10 text-current">
              <CornerDownLeft />
            </Kbd>
          </Button>
        </div>
      </div>
    </div>
  );
}

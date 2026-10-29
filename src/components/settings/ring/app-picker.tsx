import { useEffect, useRef, useState } from "react";
import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Spinner } from "@/components/ui/spinner";
import { useSettingsContext } from "@/components/settings-provider";
import type { RingItem } from "@/lib/api";
import { installedAppItem, pickApp } from "@/lib/ring";
import { AppIcon, GenericAppIcon } from "./ring-icon";
import { useInstalledApps } from "./use-installed-apps";

const ICON_PLACEHOLDER_SIZE = "size-5";

function LazyAppIcon({ path }: { path: string }) {
  const ref = useRef<HTMLSpanElement>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const node = ref.current;
    if (!node) return;
    const observer = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      setVisible(true);
      observer.disconnect();
    });
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  return (
    <span ref={ref} className={`flex shrink-0 items-center justify-center ${ICON_PLACEHOLDER_SIZE}`}>
      {visible ? <AppIcon path={path} /> : <GenericAppIcon />}
    </span>
  );
}

function AppPicker({ open, onChoose }: { open: boolean; onChoose: (item: RingItem | null) => void }) {
  const { t } = useSettingsContext();
  const { apps, failed, loading } = useInstalledApps(open);

  const browse = async () => {
    const item = await pickApp(t.exeFiles);
    if (item) onChoose(item);
  };

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onChoose(null)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t.appPickerTitle}</DialogTitle>
        </DialogHeader>
        <Button variant="outline" onClick={browse}>
          <FolderOpen />
          {t.appPickerBrowse}
        </Button>
        {failed ? (
          <p className="text-sm text-muted-foreground">{t.appPickerFailed}</p>
        ) : loading ? (
          <div className="flex items-center justify-center gap-2 py-6 text-sm text-muted-foreground">
            <Spinner />
            {t.appPickerLoading}
          </div>
        ) : (
          <Command className="border">
            <CommandInput placeholder={t.appPickerSearch} />
            <CommandList>
              <CommandEmpty>{t.appPickerEmpty}</CommandEmpty>
              {apps?.map((app) => (
                <CommandItem key={app.path} value={app.name} onSelect={() => onChoose(installedAppItem(app))}>
                  <LazyAppIcon path={app.path} />
                  <span className="truncate">{app.name}</span>
                </CommandItem>
              ))}
            </CommandList>
          </Command>
        )}
      </DialogContent>
    </Dialog>
  );
}

export function useAppPicker() {
  const [open, setOpen] = useState(false);
  const resolver = useRef<((item: RingItem | null) => void) | null>(null);

  const finish = (item: RingItem | null) => {
    resolver.current?.(item);
    resolver.current = null;
    setOpen(false);
  };

  const pick = () =>
    new Promise<RingItem | null>((resolve) => {
      resolver.current = resolve;
      setOpen(true);
    });

  return { pick, element: <AppPicker open={open} onChoose={finish} /> };
}

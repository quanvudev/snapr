import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryItem } from "@/lib/api";
import { fileName } from "@/lib/gallery";
import { defaultTarget, languageName, targets } from "@/lib/languages";

type Props = {
  target: GalleryItem | null;
  onClose: () => void;
  onTranslate: (item: GalleryItem, target: string) => void;
};

export function TranslateDialog({ target, onClose, onTranslate }: Props) {
  const { t } = useSettingsContext();
  return (
    <Dialog open={!!target} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t.translateSubtitles}</DialogTitle>
          {target && <DialogDescription>{fileName(target.path)}</DialogDescription>}
        </DialogHeader>
        {target && <TranslateForm key={target.path} item={target} onClose={onClose} onTranslate={onTranslate} />}
      </DialogContent>
    </Dialog>
  );
}

function TranslateForm({ item, onClose, onTranslate }: { item: GalleryItem } & Omit<Props, "target">) {
  const { cfg, t } = useSettingsContext();
  const [code, setCode] = useState(() => defaultTarget(item.language, cfg.language));
  const source = item.language ? languageName(item.language, cfg.language) : null;
  return (
    <>
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm">{t.translateInto}</span>
        <Select value={code} onValueChange={(v) => v && setCode(v as string)}>
          <SelectTrigger className="w-56">
            <SelectValue>{(v: string) => languageName(v, cfg.language)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {targets(item.language).map((language) => (
              <SelectItem key={language.code} value={language.code}>
                {languageName(language.code, cfg.language)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {source && <p className="text-sm text-muted-foreground">{t.translateFrom.replace("{language}", source)}</p>}
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t.cancel}
        </Button>
        <Button onClick={() => onTranslate(item, code)}>{t.translate}</Button>
      </DialogFooter>
    </>
  );
}

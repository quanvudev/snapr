import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Spinner } from "@/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type CompressHeight, type CompressInfo, type GalleryItem, type Quality } from "@/lib/api";
import { fileName, formatSize } from "@/lib/gallery";
import {
  COMPRESS_DEFAULT,
  COMPRESS_HEIGHTS,
  compressEstimate,
  compressHeightAvailable,
  loadCompressChoice,
  QUALITIES,
  type CompressChoice,
} from "@/lib/queue";

type Props = {
  item: GalleryItem | null;
  onClose: () => void;
  onCompress: (item: GalleryItem, choice: CompressChoice) => void;
  onError: (error: unknown) => void;
};

const heightKey = (height: CompressHeight) => String(height);
const parseHeight = (key: string): CompressHeight => (key === "null" ? null : (Number(key) as CompressHeight));

export function CompressDialog({ item, onClose, onCompress, onError }: Props) {
  const { t } = useSettingsContext();
  return (
    <Dialog open={!!item} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t.compressVideo}</DialogTitle>
        </DialogHeader>
        {item && <CompressForm key={item.path} item={item} onClose={onClose} onCompress={onCompress} onError={onError} />}
      </DialogContent>
    </Dialog>
  );
}

type FormProps = Omit<Props, "item"> & { item: GalleryItem };

function CompressForm({ item, onClose, onCompress, onError }: FormProps) {
  const { t } = useSettingsContext();
  const [info, setInfo] = useState<CompressInfo | null>(null);
  const [stored, setStored] = useState(loadCompressChoice);
  const failed = useRef(onError);
  failed.current = onError;

  useEffect(() => {
    let current = true;
    api
      .compressInfo(item.path)
      .then((value) => current && setInfo(value))
      .catch((e) => current && failed.current(e));
    return () => {
      current = false;
    };
  }, [item.path]);

  const choice: CompressChoice =
    info && !compressHeightAvailable(info, stored.height) ? { ...stored, height: COMPRESS_DEFAULT.height } : stored;
  const estimate = info ? compressEstimate(info, choice) : null;

  return (
    <>
      <div className="grid gap-1">
        <span className="truncate font-medium">{fileName(item.path)}</span>
        <span className="text-xs text-muted-foreground">
          {t.compressCurrentSize.replace("{size}", formatSize(item.size))}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <div className="flex items-center gap-2">
          <span>{t.compressQuality}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[choice.quality]}
            onValueChange={(v) => v[0] && setStored({ ...choice, quality: v[0] as Quality })}
          >
            {QUALITIES.map((q) => (
              <ToggleGroupItem key={q.value} value={q.value}>
                {t[q.label]}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
        <div className="flex items-center gap-2">
          <span>{t.compressResolution}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[heightKey(choice.height)]}
            onValueChange={(v) => v[0] && setStored({ ...choice, height: parseHeight(v[0]) })}
          >
            {COMPRESS_HEIGHTS.map((height) => (
              <ToggleGroupItem
                key={heightKey(height)}
                value={heightKey(height)}
                disabled={!info || !compressHeightAvailable(info, height)}
              >
                {height ? `${height}p` : t.gifOriginal}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
      </div>
      <div className="min-h-4 text-xs text-muted-foreground">
        {info ? estimate !== null && t.compressEstimate.replace("{size}", formatSize(estimate)) : <Spinner />}
      </div>
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t.cancel}
        </Button>
        <Button disabled={!info} onClick={() => onCompress(item, choice)}>
          {t.compressAction}
        </Button>
      </DialogFooter>
    </>
  );
}

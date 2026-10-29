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
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryItem } from "@/lib/api";
import { fileName } from "@/lib/gallery";

type Props = { items: GalleryItem[] | null; onCancel: () => void; onConfirm: () => void };

export function DeleteDialog({ items, onCancel, onConfirm }: Props) {
  const { t } = useSettingsContext();
  const count = items?.length ?? 0;
  const single = count === 1 ? items?.[0] : null;
  return (
    <AlertDialog open={count > 0} onOpenChange={(open) => !open && onCancel()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{single ? t.deleteTitle : t.bulkDeleteTitle}</AlertDialogTitle>
          <AlertDialogDescription>
            {single ? `${t.deleteHint} ${fileName(single.path)}` : t.bulkDeleteHint.replace("{count}", String(count))}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            {single ? t.deleteItem : t.bulkDeleteAction}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

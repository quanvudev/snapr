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

type Props = { item: GalleryItem | null; onCancel: () => void; onConfirm: () => void };

export function DeleteDialog({ item, onCancel, onConfirm }: Props) {
  const { t } = useSettingsContext();
  return (
    <AlertDialog open={!!item} onOpenChange={(open) => !open && onCancel()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t.deleteTitle}</AlertDialogTitle>
          <AlertDialogDescription>
            {t.deleteHint} {item && fileName(item.path)}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            {t.deleteItem}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

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
import { Button } from "@/components/ui/button";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { HYMT_LICENSE_NAME, HYMT_LICENSE_URL, HYMT_NAME } from "@/lib/hymt";
import { mb } from "./whisper-models";

type Props = { open: boolean; bytes: number; onClose: () => void };

export function LicenseDialog({ open, bytes, onClose }: Props) {
  const { cfg, t, update } = useSettingsContext();
  const agree = () => {
    onClose();
    update({ ...cfg, hymt_license_accepted: true });
  };
  return (
    <AlertDialog open={open} onOpenChange={(next) => !next && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t.hymtLicenseTitle.replace("{name}", HYMT_NAME)}</AlertDialogTitle>
          <AlertDialogDescription>{t.hymtLicenseSize.replace("{bytes}", mb(bytes))}</AlertDialogDescription>
        </AlertDialogHeader>
        <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
          <li>{t.hymtLicenseTerms.replace("{license}", HYMT_LICENSE_NAME)}</li>
          <li>{t.hymtLicenseTerritory}</li>
          <li>{t.hymtLicenseOutput}</li>
          <li>{t.hymtLicenseAffiliation}</li>
        </ul>
        <Button variant="link" className="self-start px-0" onClick={() => api.openUrl(HYMT_LICENSE_URL).catch(() => {})}>
          {t.hymtLicenseRead}
        </Button>
        <AlertDialogFooter>
          <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
          <AlertDialogAction onClick={agree}>{t.hymtLicenseAgree}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

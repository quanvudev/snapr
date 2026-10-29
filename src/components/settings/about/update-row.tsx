import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Progress, ProgressValue } from "@/components/ui/progress";
import { Spinner } from "@/components/ui/spinner";
import { useSettingsContext } from "@/components/settings-provider";

export function UpdateRow() {
  const { t, updater } = useSettingsContext();
  const { status, check, install } = updater;
  const version = status.version ?? "";
  const percent = status.total > 0 ? (status.done / status.total) * 100 : 0;
  const busy = (label: string) => (
    <span className="flex items-center gap-2 text-sm text-muted-foreground">
      <Spinner />
      {label}
    </span>
  );
  const action = (label: string, onClick: () => void) => (
    <Button size="sm" variant="outline" onClick={onClick}>
      {label}
    </Button>
  );

  const description = {
    idle: null,
    checking: null,
    latest: t.updateLatest,
    available: t.updateAvailable.replace("{version}", version),
    blocked: t.updateBlocked,
    downloading: null,
    installing: null,
    error: t.updateFailed.replace("{error}", status.error ?? ""),
  }[status.state];

  const actions = {
    idle: action(t.updateCheckNow, check),
    checking: busy(t.updateChecking),
    latest: action(t.updateCheckNow, check),
    available: action(t.updateInstall, install),
    blocked: action(t.updateInstall, install),
    downloading: null,
    installing: busy(t.updateInstalling),
    error: action(t.paddleRetry, status.version ? install : check),
  }[status.state];

  return (
    <Item variant="muted" className="flex-wrap">
      <ItemContent>
        <ItemTitle>{t.updates}</ItemTitle>
        {description && (
          <ItemDescription className={status.state === "error" ? "text-destructive" : undefined}>{description}</ItemDescription>
        )}
      </ItemContent>
      {actions && <ItemActions>{actions}</ItemActions>}
      {status.state === "downloading" && (
        <Progress value={percent} className="basis-full">
          <span className="text-sm">{t.updateDownloading.replace("{version}", version)}</span>
          <ProgressValue>{() => `${Math.round(percent)}%`}</ProgressValue>
        </Progress>
      )}
    </Item>
  );
}

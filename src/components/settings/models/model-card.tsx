import { Button } from "@/components/ui/button";
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemTitle,
} from "@/components/ui/item";
import { Progress, ProgressValue } from "@/components/ui/progress";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { PERCENT } from "@/lib/queue";
import { labelOf, type ModelStore } from "./stores";

const SELECT_AFTER = 4;

type Props = {
  title: string;
  description?: string;
  store: ModelStore;
  children?: React.ReactNode;
};

export function ModelCard({ title, description, store, children }: Props) {
  const { t } = useSettingsContext();
  const { progress, error, selected } = store;
  const failed = error?.id === selected ? error.message : null;
  const ready = store.ready.includes(selected);
  const percent =
    progress && progress.total > 0
      ? (progress.done / progress.total) * PERCENT
      : 0;
  const change = (value: string[]) => {
    if (value[0]) store.select(value[0]);
  };
  const text = (id: string) => {
    const option = store.options.find((o) => o.id === id);
    return option
      ? option.detail
        ? `${option.label} · ${option.detail}`
        : option.label
      : id;
  };

  return (
    <Item variant="muted" className="flex-wrap">
      <ItemContent>
        <ItemTitle>{title}</ItemTitle>
        {description && <ItemDescription>{description}</ItemDescription>}
      </ItemContent>
      <ItemActions>
        {store.options.length > SELECT_AFTER ? (
          <Select
            value={selected}
            onValueChange={(v) => v && store.select(v as string)}
          >
            <SelectTrigger className="w-56">
              <SelectValue>{(v: string) => text(v)}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              {store.options.map((option) => (
                <SelectItem key={option.id} value={option.id}>
                  {text(option.id)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <ToggleGroup
            variant="outline"
            size="sm"
            value={[selected]}
            onValueChange={change}
          >
            {store.options.map((option) => (
              <ToggleGroupItem
                key={option.id}
                value={option.id}
                className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
              >
                {option.label}
                {option.detail && ` · ${option.detail}`}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        )}
      </ItemActions>
      {children}
      {progress ? (
        <div className="flex basis-full items-center gap-3">
          <Progress value={percent} className="flex-1">
            <span className="text-sm">
              {t.modelDownloading.replace(
                "{name}",
                progress.label ?? labelOf(store, progress.id),
              )}
            </span>
            <ProgressValue>{() => `${Math.round(percent)}%`}</ProgressValue>
          </Progress>
          <Button size="sm" variant="outline" onClick={store.cancel}>
            {t.cancel}
          </Button>
        </div>
      ) : failed ? (
        <div className="flex basis-full items-center justify-between gap-3">
          <span className="text-sm text-destructive">
            {t.paddleFailed.replace("{error}", failed)}
          </span>
          <Button
            size="sm"
            variant="outline"
            onClick={() => store.download(selected)}
          >
            {t.paddleRetry}
          </Button>
        </div>
      ) : (
        !ready && (
          <div className="flex basis-full items-center justify-between gap-3">
            <span className="text-sm text-muted-foreground">
              {t.paddleNotDownloaded}
            </span>
            <Button
              size="sm"
              variant="outline"
              onClick={() => store.download(selected)}
            >
              {t.paddleDownload}
            </Button>
          </div>
        )
      )}
    </Item>
  );
}

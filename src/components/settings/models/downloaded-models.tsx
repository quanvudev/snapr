import { useState } from "react";
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
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Empty, EmptyHeader, EmptyTitle } from "@/components/ui/empty";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { SectionHeading } from "../section-heading";
import { SettingsCard } from "../settings-card";
import { labelOf, type ModelStore } from "./stores";
import { mb } from "./whisper-models";

type Target = { store: ModelStore; id: string };

const rowName = (store: ModelStore, id: string) =>
  labelOf(store, id) === store.name ? store.name : `${store.name} · ${labelOf(store, id)}`;

export function DownloadedModels({ stores }: { stores: ModelStore[] }) {
  const { t } = useSettingsContext();
  const [target, setTarget] = useState<Target | null>(null);
  const rows = stores.flatMap((store) => store.options.filter((o) => store.ready.includes(o.id)).map((o) => ({ store, id: o.id })));
  const total = rows.reduce((sum, { store, id }) => sum + store.bytes[id], 0);

  return (
    <>
      <div className="flex items-center justify-between">
        <SectionHeading>{t.modelsDownloaded}</SectionHeading>
        {rows.length > 0 && <span className="px-1 text-xs text-muted-foreground">{mb(total)}</span>}
      </div>
      {rows.length === 0 ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>{t.modelsEmpty}</EmptyTitle>
          </EmptyHeader>
        </Empty>
      ) : (
        <SettingsCard>
          {rows.map(({ store, id }) => {
            const inUse = store.selected === id;
            return (
              <Item key={`${store.name}-${id}`} variant="muted" size="sm">
                <ItemContent>
                  <ItemTitle>
                    {rowName(store, id)}
                    <Badge variant={inUse ? "default" : "secondary"}>{inUse ? t.paddleInUse : t.paddleDownloaded}</Badge>
                  </ItemTitle>
                </ItemContent>
                <ItemActions className="gap-3">
                  <span className="text-sm text-muted-foreground">{mb(store.bytes[id])}</span>
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={store.progress?.id === id}
                    onClick={() => setTarget({ store, id })}
                  >
                    {t.paddleDelete}
                  </Button>
                </ItemActions>
              </Item>
            );
          })}
        </SettingsCard>
      )}
      <AlertDialog open={target !== null} onOpenChange={(open) => !open && setTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {target && t.paddleDeleteTitle.replace("{name}", rowName(target.store, target.id))}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {target && t.paddleDeleteHint.replace("{bytes}", mb(target.store.bytes[target.id]))}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (target) target.store.remove(target.id);
                setTarget(null);
              }}
            >
              {t.paddleDelete}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

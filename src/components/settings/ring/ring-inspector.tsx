import { ArrowLeft, ArrowRight, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Card, CardAction, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useSettingsContext } from "@/components/settings-provider";
import type { RingItem } from "@/lib/api";
import { isStoreApp, itemName, pickApp, resolveKeys } from "@/lib/ring";
import { CommitInput } from "./commit-input";
import { KeyRecorder } from "./key-recorder";
import { RingIcon } from "./ring-icon";

type App = Extract<RingItem, { kind: "app" }>;

type Props = {
  item: RingItem;
  items: RingItem[];
  index: number;
  count: number;
  onChange: (item: RingItem) => void;
  onMove: (to: number) => void;
  onRemove: () => void;
  onRecordingKey: (recording: boolean) => void;
};

function AppFields({ item, onChange }: { item: App; onChange: (item: RingItem) => void }) {
  const { t } = useSettingsContext();
  const store = isStoreApp(item.path);

  const browse = async () => {
    const picked = await pickApp(t.exeFiles);
    if (picked?.kind === "app") onChange({ ...item, path: picked.path });
  };

  return (
    <>
      <Field>
        <FieldLabel>{t.ringLabel}</FieldLabel>
        <CommitInput value={item.label} onCommit={(label) => onChange({ ...item, label })} />
      </Field>
      <Field>
        <FieldLabel>{t.ringPath}</FieldLabel>
        <div className="flex gap-2">
          <Input readOnly value={item.path} title={item.path} />
          <Button variant="outline" onClick={browse}>
            {t.browse}
          </Button>
        </div>
      </Field>
      <Field>
        <FieldLabel>{t.ringArgs}</FieldLabel>
        <CommitInput value={item.args} onCommit={(args) => onChange({ ...item, args })} />
      </Field>
      <Field orientation="horizontal">
        <Checkbox
          id="ring-admin"
          checked={item.admin && !store}
          disabled={store}
          onCheckedChange={(admin) => onChange({ ...item, admin })}
        />
        <FieldLabel htmlFor="ring-admin">{t.ringAdmin}</FieldLabel>
      </Field>
      {store && <p className="text-xs text-muted-foreground">{t.ringAdminHint}</p>}
    </>
  );
}

function KeyField({ item, items, index, onChange, onRecordingKey }: Pick<Props, "item" | "items" | "index" | "onChange" | "onRecordingKey">) {
  const { t } = useSettingsContext();
  const rival = item.key ? items.find((other, i) => i !== index && other.key === item.key) : undefined;

  return (
    <Field>
      <FieldLabel>{t.ringKey}</FieldLabel>
      <KeyRecorder
        configured={item.key}
        auto={resolveKeys(items)[index]}
        onChange={(key) => onChange({ ...item, key })}
        onRecording={onRecordingKey}
      />
      {rival && (
        <p className="text-xs text-destructive">
          {t.ringKeyConflict} {itemName(rival, t)}
        </p>
      )}
    </Field>
  );
}

function ActionButton({
  label,
  variant = "ghost",
  disabled,
  onClick,
  children,
}: {
  label: string;
  variant?: "ghost" | "destructive";
  disabled?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={<Button variant={variant} size="icon-sm" aria-label={label} disabled={disabled} onClick={onClick} />}
      >
        {children}
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

export function RingInspector({ item, items, index, count, onChange, onMove, onRemove, onRecordingKey }: Props) {
  const { cfg, t, textActive } = useSettingsContext();
  const name = itemName(item, t);
  const showText = item.kind === "builtin" && item.id === "text" && !textActive;

  return (
    <Card size="sm">
      <CardHeader className="flex items-center justify-between gap-2">
        <CardTitle className="flex min-w-0 items-center gap-2">
          <RingIcon item={item} />
          <span className="truncate">{name}</span>
        </CardTitle>
        <CardAction className="flex items-center gap-1">
          <ActionButton label={t.ringMoveEarlier} disabled={index === 0} onClick={() => onMove(index - 1)}>
            <ArrowLeft />
          </ActionButton>
          <ActionButton label={t.ringMoveLater} disabled={index === count - 1} onClick={() => onMove(index + 1)}>
            <ArrowRight />
          </ActionButton>
          <ActionButton label={t.ringRemove} variant="destructive" onClick={onRemove}>
            <Trash2 />
          </ActionButton>
        </CardAction>
      </CardHeader>
      {(item.kind === "app" || showText || cfg.ring.keys) && (
        <CardContent className="space-y-4">
          {cfg.ring.keys && (
            <KeyField item={item} items={items} index={index} onChange={onChange} onRecordingKey={onRecordingKey} />
          )}
          {item.kind === "app" && <AppFields item={item} onChange={onChange} />}
          {showText && <p className="text-xs text-muted-foreground">{t.ringTextHint}</p>}
        </CardContent>
      )}
    </Card>
  );
}

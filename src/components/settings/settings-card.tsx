import { Children, Fragment } from "react";
import { ItemSeparator } from "@/components/ui/item";

export function SettingsCard({ children }: { children: React.ReactNode }) {
  return (
    <div role="list" className="rounded-md bg-muted/50 [&_[data-slot=item]]:bg-transparent">
      {Children.toArray(children).map((row, i) => (
        <Fragment key={i}>
          {i > 0 && <ItemSeparator className="my-0" />}
          {row}
        </Fragment>
      ))}
    </div>
  );
}

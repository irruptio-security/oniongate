import {
  cloneElement,
  isValidElement,
  useId,
  type ReactElement,
  type ReactNode,
} from "react";
import { cn } from "@/lib/utils";

export function SettingRow({
  title,
  description,
  children,
  className,
}: {
  title: ReactNode;
  description?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  const titleId = useId();
  // Give the control an accessible name from the row title (Radix Switch/Select
  // otherwise have no label). Existing aria-labelledby is preserved.
  const labelledControl = isValidElement(children)
    ? cloneElement(children as ReactElement<{ "aria-labelledby"?: string }>, {
        "aria-labelledby":
          (children.props as { "aria-labelledby"?: string })[
            "aria-labelledby"
          ] ?? titleId,
      })
    : children;

  return (
    <div
      className={cn(
        "flex items-center justify-between gap-3 rounded-xl border border-line bg-panel px-3.5 py-3",
        className,
      )}
    >
      <div className="min-w-0">
        <div id={titleId} className="text-sm font-semibold text-ink">
          {title}
        </div>
        {description ? (
          <p className="mt-0.5 text-xs leading-snug text-muted">{description}</p>
        ) : null}
      </div>
      <div className="shrink-0">{labelledControl}</div>
    </div>
  );
}

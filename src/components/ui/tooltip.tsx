import * as React from "react";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { Info } from "lucide-react";
import { cn } from "@/lib/utils";

export function TooltipProvider({
  children,
  delayDuration = 200,
}: {
  children: React.ReactNode;
  delayDuration?: number;
}) {
  return (
    <TooltipPrimitive.Provider delayDuration={delayDuration}>
      {children}
    </TooltipPrimitive.Provider>
  );
}

export type TipTone = "default" | "ok" | "warn" | "danger" | "accent";

const chipTone: Record<TipTone, string> = {
  default: "bg-panel text-muted border-line",
  ok: "bg-accent/15 text-accent-strong border-accent/35",
  warn: "bg-warn/15 text-warn-strong border-warn/35",
  danger: "bg-danger/15 text-danger-strong border-danger/35",
  accent: "bg-accent/15 text-accent-strong border-accent/35",
};

export function TipChip({
  tone = "default",
  className,
  children,
}: {
  tone?: TipTone;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
        chipTone[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}

type StatusChip = { label: React.ReactNode; tone?: TipTone };

export function InfoTip({
  content,
  title,
  description,
  status,
  detail,
  risk,
  riskTone = "warn",
  action,
  side = "top",
  className,
}: {
  /** Simple mode: render arbitrary node. Overrides structured fields. */
  content?: React.ReactNode;
  title?: React.ReactNode;
  description?: React.ReactNode;
  /** Short state chip(s) shown under the title. */
  status?: StatusChip | StatusChip[];
  /** Longer status line (e.g. probe output). */
  detail?: React.ReactNode;
  /** Highlighted risk callout at the bottom. */
  risk?: React.ReactNode;
  riskTone?: "warn" | "danger";
  /** Optional action button rendered at the bottom of the popup. */
  action?: { label: React.ReactNode; onClick: () => void };
  side?: "top" | "right" | "bottom" | "left";
  className?: string;
}) {
  const chips = status
    ? Array.isArray(status)
      ? status
      : [status]
    : [];
  const hasStructured =
    title != null ||
    description != null ||
    chips.length > 0 ||
    detail != null ||
    risk != null ||
    action != null;

  if (content == null && !hasStructured) return null;

  const riskStyles =
    riskTone === "danger"
      ? { box: "border-danger/30 bg-danger/10", dot: "bg-danger", label: "text-danger" }
      : { box: "border-warn/30 bg-warn/10", dot: "bg-warn", label: "text-warn" };

  const body = content ?? (
    <div className="space-y-2">
      {title != null ? (
        <div className="text-[13px] font-semibold leading-tight text-ink">
          {title}
        </div>
      ) : null}
      {chips.length ? (
        <div className="flex flex-wrap gap-1">
          {chips.map((c, i) => (
            <TipChip key={i} tone={c.tone}>
              {c.label}
            </TipChip>
          ))}
        </div>
      ) : null}
      {description != null ? (
        <p className="text-[11px] leading-relaxed text-muted">{description}</p>
      ) : null}
      {detail != null ? (
        <div className="rounded-md border border-line bg-panel px-2 py-1 text-[11px] leading-relaxed text-ink">
          {detail}
        </div>
      ) : null}
      {risk != null ? (
        <div
          className={cn(
            "flex items-start gap-1.5 rounded-md border px-2 py-1.5",
            riskStyles.box,
          )}
        >
          <span
            className={cn(
              "mt-[4px] h-1.5 w-1.5 shrink-0 rounded-full",
              riskStyles.dot,
            )}
          />
          <span className="text-[11px] leading-relaxed text-ink">
            <span className={cn("font-semibold", riskStyles.label)}>Risk · </span>
            {risk}
          </span>
        </div>
      ) : null}
      {action != null ? (
        <button
          type="button"
          onClick={action.onClick}
          className="inline-flex items-center gap-1 rounded-md border border-line bg-panel-2 px-2 py-1 text-[11px] font-semibold text-ink transition-colors hover:border-onion/60 hover:text-onion"
        >
          {action.label}
        </button>
      ) : null}
    </div>
  );

  return (
    <TooltipPrimitive.Root>
      <TooltipPrimitive.Trigger asChild>
        <button
          type="button"
          className={cn(
            "inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-muted transition-colors hover:text-ink focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
            className,
          )}
          aria-label="More info"
        >
          <Info className="h-3.5 w-3.5" strokeWidth={2} />
        </button>
      </TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content
          side={side}
          sideOffset={6}
          collisionPadding={12}
          className="z-50 w-72 max-w-[calc(100vw-24px)] rounded-xl border border-line bg-panel p-3 text-xs leading-relaxed text-ink shadow-xl animate-[fade-in_120ms_ease-out]"
        >
          {body}
          <TooltipPrimitive.Arrow className="fill-[var(--color-panel)]" />
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
}

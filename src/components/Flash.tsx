import { useEffect } from "react";
import { CheckCircle2, CircleAlert, X } from "lucide-react";
import { cn } from "@/lib/utils";

export function Flash({
  message,
  error,
  onDismiss,
}: {
  message?: string | null;
  error?: string | null;
  onDismiss?: () => void;
}) {
  const text = error || message;
  const kind = error ? "error" : message ? "ok" : null;

  useEffect(() => {
    if (!kind || !onDismiss) return;
    const ms = kind === "error" ? 8000 : 4200;
    const id = window.setTimeout(onDismiss, ms);
    return () => window.clearTimeout(id);
  }, [kind, text, onDismiss]);

  if (!kind || !text) return null;

  return (
    <div
      className="pointer-events-none absolute bottom-4 right-4 z-[100] flex max-w-[min(22rem,calc(100%-2rem))] justify-end"
      role="status"
      aria-live="polite"
    >
      <div
        className={cn(
          "pointer-events-auto flex w-full items-start gap-2.5 rounded-xl border px-3.5 py-3 shadow-lg shadow-ink/10 backdrop-blur-md animate-[toast-in_180ms_ease-out]",
          kind === "ok"
            ? "border-accent/45 bg-panel text-ink"
            : "border-danger/45 bg-panel text-ink",
        )}
      >
        {kind === "ok" ? (
          <CheckCircle2
            className="mt-0.5 h-4 w-4 shrink-0 text-accent"
            strokeWidth={2}
          />
        ) : (
          <CircleAlert
            className="mt-0.5 h-4 w-4 shrink-0 text-danger"
            strokeWidth={2}
          />
        )}
        <div className="min-w-0 flex-1">
          <div
            className={cn(
              "text-[11px] font-semibold uppercase tracking-wide",
              kind === "ok" ? "text-accent" : "text-danger",
            )}
          >
            {kind === "ok" ? "Success" : "Failed"}
          </div>
          <p className="mt-0.5 text-sm leading-snug text-ink">{text}</p>
        </div>
        {onDismiss ? (
          <button
            type="button"
            onClick={onDismiss}
            className="shrink-0 rounded-md p-1 text-muted transition-colors hover:bg-panel-2 hover:text-ink"
            aria-label="Dismiss"
          >
            <X className="h-3.5 w-3.5" strokeWidth={2} />
          </button>
        ) : null}
      </div>
    </div>
  );
}

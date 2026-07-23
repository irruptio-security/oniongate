import { Button } from "@/components/ui/button";

/**
 * Shared loading / error state for pages that depend on async bootstrap data.
 * Shows a spinner-free "Loading…" label, or an error with a Retry action so a
 * failed bootstrap never leaves the page stuck on "Loading…" forever.
 */
export function LoadState({
  label = "Loading…",
  error,
  onRetry,
}: {
  label?: string;
  error?: string | null;
  onRetry?: () => void;
}) {
  if (error) {
    return (
      <div className="flex flex-col items-start gap-2 rounded-xl border border-line bg-panel px-4 py-3">
        <div className="text-sm font-semibold">Couldn't load</div>
        <p className="text-xs text-muted">{error}</p>
        {onRetry ? (
          <Button size="sm" variant="secondary" onClick={onRetry}>
            Retry
          </Button>
        ) : null}
      </div>
    );
  }
  return <p className="text-sm text-muted">{label}</p>;
}

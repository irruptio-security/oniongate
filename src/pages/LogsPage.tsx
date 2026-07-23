import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { TorApp } from "@/hooks/useTorApp";

export function LogsPage({ app }: { app: TorApp }) {
  const { logs, busy, run, refreshLogs } = app;

  // Logs is a System sub-view now; poll while this page is mounted.
  useEffect(() => {
    void refreshLogs();
    const id = window.setInterval(() => void refreshLogs(), 2000);
    return () => window.clearInterval(id);
  }, [refreshLogs]);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-start justify-between gap-2">
        <div>
          <h2 className="text-xl font-semibold tracking-tight">Logs</h2>
          <p className="mt-1 text-sm text-muted">
            {logs?.source ?? "Tor + app events"}
            {logs?.log_path ? ` · ${logs.log_path}` : ""}
          </p>
        </div>
        <div className="flex gap-1">
          <Button
            size="icon"
            variant="ghost"
            disabled={busy}
            aria-label="Refresh logs"
            onClick={() =>
              void run(async () => {
                await refreshLogs();
                return "Logs refreshed";
              })
            }
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={busy}
            onClick={() =>
              void run(async () => {
                const msg = await invoke<string>("clear_tor_logs");
                await refreshLogs();
                return msg;
              })
            }
          >
            Clear
          </Button>
        </div>
      </div>

      <div className="flex min-h-[420px] flex-1 flex-col overflow-hidden rounded-xl border border-line bg-canvas">
        <div className="flex items-center gap-1.5 border-b border-line px-3 py-2">
          <span className="h-2.5 w-2.5 rounded-full bg-danger/70" />
          <span className="h-2.5 w-2.5 rounded-full bg-warn/70" />
          <span className="h-2.5 w-2.5 rounded-full bg-accent/70" />
          <span className="ml-2 font-mono text-[11px] text-muted">tor</span>
        </div>
        <pre className="m-0 h-[380px] overflow-auto p-3 font-mono text-[11px] leading-relaxed text-muted whitespace-pre-wrap">
          {(logs?.lines ?? ["Loading…"]).join("\n")}
        </pre>
      </div>
    </section>
  );
}

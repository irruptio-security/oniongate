import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { TorApp } from "@/hooks/useTorApp";
import { cn } from "@/lib/utils";

type VerificationCheck = {
  id: string;
  label: string;
  status: "pass" | "warn" | "fail";
  detail: string;
  remediation: string | null;
};

type LeakReport = {
  created_at_unix: number;
  passed: boolean;
  checks: VerificationCheck[];
};

type OnionResult = {
  reachable: boolean;
  latency_ms: number | null;
  detail: string;
};

export function VerifyPage({ app }: { app: TorApp }) {
  const { busy, run } = app;
  const [report, setReport] = useState<LeakReport | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [onionHost, setOnionHost] = useState("");
  const [onionResult, setOnionResult] = useState<OnionResult | null>(null);

  return (
    <section className="flex flex-col gap-5">
      <header>
        <h2 className="text-xl font-semibold tracking-tight">Verify</h2>
        <p className="mt-1 text-sm text-muted">
          Test Tor egress, DNS, IPv6, UDP/QUIC containment, app policy, and recovery state.
          Public addresses are compared in memory and never stored.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        <Button
          disabled={busy}
          onClick={async () => {
            setVerifying(true);
            try {
              await run(async () => {
                const next = await invoke<LeakReport>("run_leak_verifier");
                setReport(next);
                return next.passed
                  ? "Verification passed"
                  : "Verification found one or more failures";
              });
            } finally {
              setVerifying(false);
            }
          }}
        >
          {verifying ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              Running checks…
            </>
          ) : (
            "Run leak verifier"
          )}
        </Button>
        <Button
          variant="secondary"
          disabled={busy || !report}
          onClick={() =>
            void run(async () => {
              const path = await save({
                defaultPath: "oniongate-verification.json",
                filters: [{ name: "JSON", extensions: ["json"] }],
              });
              if (!path) return "Export cancelled";
              return invoke<string>("export_latest_leak_report", { path });
            })
          }
        >
          Export redacted report
        </Button>
      </div>

      {!report ? (
        <div className="rounded-xl border border-dashed border-line bg-panel/60 px-4 py-6 text-center">
          <div className="text-sm font-semibold">No verification run yet</div>
          <p className="mx-auto mt-1 max-w-md text-xs text-muted">
            Connect through Tor, then run the leak verifier to check egress
            separation, DNS, IPv6, UDP/QUIC containment, app policy, and recovery
            state. Results stay on this device.
          </p>
        </div>
      ) : null}

      {report ? (
        <div className="space-y-1.5">
          {report.checks.map((item) => (
            <div
              key={item.id}
              className="flex items-start justify-between gap-4 rounded-xl border border-line bg-panel px-3.5 py-2.5"
            >
              <div className="min-w-0">
                <div className="text-sm font-semibold">{item.label}</div>
                <div className="text-xs text-muted">{item.detail}</div>
                {item.status !== "pass" && item.remediation ? (
                  <div
                    className={cn(
                      "mt-1.5 rounded-md border px-2 py-1.5 text-[11px] leading-relaxed",
                      item.status === "fail"
                        ? "border-danger/30 bg-danger/10 text-ink"
                        : "border-warn/30 bg-warn/10 text-ink",
                    )}
                  >
                    <span
                      className={cn(
                        "font-semibold",
                        item.status === "fail"
                          ? "text-danger-strong"
                          : "text-warn-strong",
                      )}
                    >
                      How to fix ·{" "}
                    </span>
                    {item.remediation}
                  </div>
                ) : null}
              </div>
              <span
                className={cn(
                  "shrink-0 rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase",
                  item.status === "pass" && "bg-accent/15 text-accent-strong",
                  item.status === "warn" && "bg-warn/15 text-warn-strong",
                  item.status === "fail" && "bg-danger/15 text-danger-strong",
                )}
              >
                {item.status}
              </span>
            </div>
          ))}
        </div>
      ) : null}

      <div className="rounded-xl border border-line bg-panel p-4">
        <div className="text-sm font-semibold">Test a v3 onion service</div>
        <p className="mt-0.5 text-xs text-muted">
          Uses a SOCKS5 domain request, proving the hostname was sent to Tor instead of local DNS.
        </p>
        <div className="mt-3 flex gap-2">
          <Input
            value={onionHost}
            disabled={busy}
            placeholder="56-character-address.onion"
            onChange={(event) => setOnionHost(event.target.value)}
          />
          <Button
            variant="secondary"
            disabled={busy || !onionHost.trim()}
            onClick={() =>
              void run(async () => {
                const result = await invoke<OnionResult>("test_onion_connectivity", {
                  host: onionHost,
                  port: 80,
                });
                setOnionResult(result);
                return result.detail;
              })
            }
          >
            Test
          </Button>
        </div>
        {onionResult ? (
          <p className={cn("mt-2 text-xs", onionResult.reachable ? "text-accent" : "text-danger")}>
            {onionResult.detail}
            {onionResult.latency_ms != null ? ` · ${onionResult.latency_ms} ms` : ""}
          </p>
        ) : null}
      </div>
    </section>
  );
}

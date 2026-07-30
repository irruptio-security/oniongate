import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  AppStatus,
  BridgeScanResult,
  DetectReport,
  ExitCountryOption,
  FetchBridgesResult,
  HardenItem,
  IpReport,
  NetworkTestResult,
  NewIdentityResult,
  RelayInfo,
  SessionOverview,
  ShellProxyStatus,
  SnowflakeStatus,
  SystemView,
  Tab,
  TorLogs,
} from "@/lib/types";

export function useTorApp() {
  const [tab, setTab] = useState<Tab>("home");
  const [systemView, setSystemView] = useState<SystemView>("audit");
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [ips, setIps] = useState<IpReport | null>(null);
  const [detect, setDetect] = useState<DetectReport | null>(null);
  const [shellProxy, setShellProxy] = useState<ShellProxyStatus | null>(null);
  const [networkTest, setNetworkTest] = useState<NetworkTestResult | null>(
    null,
  );
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [logs, setLogs] = useState<TorLogs | null>(null);
  const [bridgeText, setBridgeText] = useState("");
  const [catalogBridges, setCatalogBridges] = useState<string[]>([]);
  const [scanResults, setScanResults] = useState<BridgeScanResult[] | null>(
    null,
  );
  const [exitDraft, setExitDraft] = useState("");
  const [exitCountries, setExitCountries] = useState<ExitCountryOption[]>([]);
  const [relayQuery, setRelayQuery] = useState("");
  const [relays, setRelays] = useState<RelayInfo[] | null>(null);
  const [snowflake, setSnowflake] = useState<SnowflakeStatus | null>(null);
  const [harden, setHarden] = useState<HardenItem[]>([]);
  const [session, setSession] = useState<SessionOverview | null>(null);
  const [scanTransport, setScanTransport] = useState("obfs4");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    setStatus(await invoke<AppStatus>("get_status"));
  }, []);

  const refreshIps = useCallback(async () => {
    setIps(await invoke<IpReport>("refresh_ips"));
  }, []);

  const refreshDetect = useCallback(async () => {
    setDetect(await invoke<DetectReport>("detect_apps"));
  }, []);

  const refreshShellProxy = useCallback(async () => {
    setShellProxy(await invoke<ShellProxyStatus>("get_shell_proxy_status"));
  }, []);

  const refreshSettings = useCallback(async () => {
    const s = await invoke<AppSettings>("get_settings");
    setSettings(s);
    setBridgeText(s.bridge_lines.join("\n"));
    setExitDraft(s.exit_country);
  }, []);

  const refreshSnowflake = useCallback(async () => {
    setSnowflake(await invoke<SnowflakeStatus>("get_snowflake_status"));
  }, []);

  const refreshHarden = useCallback(async () => {
    setHarden(await invoke<HardenItem[]>("get_harden_items"));
  }, []);

  const refreshLogs = useCallback(async () => {
    setLogs(await invoke<TorLogs>("get_tor_logs"));
  }, []);

  const refreshSession = useCallback(async () => {
    setSession(await invoke<SessionOverview>("get_session_overview"));
  }, []);

  const clearFlash = useCallback(() => {
    setMessage(null);
    setError(null);
  }, []);

  const run = useCallback(
    async (action: () => Promise<string>, opts?: { refreshIps?: boolean }) => {
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        setMessage(await action());
        await refreshStatus();
        if (opts?.refreshIps) await refreshIps();
      } catch (e) {
        setError(typeof e === "string" ? e : String(e));
        await refreshStatus();
      } finally {
        setBusy(false);
      }
    },
    [refreshIps, refreshStatus],
  );

  const bootstrap = useCallback(async () => {
    setBootstrapError(null);
    try {
      const [st, det, s, shell, countries] = await Promise.all([
        invoke<AppStatus>("get_status"),
        invoke<DetectReport>("detect_apps"),
        invoke<AppSettings>("get_settings"),
        invoke<ShellProxyStatus>("get_shell_proxy_status"),
        invoke<ExitCountryOption[]>("exit_country_options"),
      ]);
      setStatus(st);
      setDetect(det);
      setSettings(s);
      setBridgeText(s.bridge_lines.join("\n"));
      setExitDraft(s.exit_country);
      setShellProxy(shell);
      setExitCountries(countries);
      const report = await invoke<IpReport>("refresh_ips");
      setIps(report);
    } catch (e) {
      const msg = typeof e === "string" ? e : String(e);
      setBootstrapError(msg);
      setError(msg);
    }
  }, []);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  useEffect(() => {
    const id = window.setInterval(() => {
      void refreshStatus();
      void refreshSession();
    }, (settings?.status_poll_secs ?? 4) * 1000);
    return () => window.clearInterval(id);
  }, [refreshStatus, refreshSession, settings?.status_poll_secs]);

  useEffect(() => {
    // Nav consolidated into Connect (home) + System. Network/Bridges live under
    // Connect; Settings/Logs/Harden live under System.
    if (tab === "home" || tab === "system") {
      void refreshSettings();
      void refreshSnowflake();
    }
    if (tab === "system") {
      void refreshHarden();
    }
    if (tab === "apps") {
      void refreshSettings();
      void refreshDetect();
      void refreshShellProxy();
    }
  }, [
    tab,
    refreshDetect,
    refreshSettings,
    refreshShellProxy,
    refreshSnowflake,
    refreshHarden,
  ]);

  // Apply the theme setting: toggle the `.dark` class on <html>, following the
  // OS when set to "auto".
  useEffect(() => {
    const theme = settings?.theme ?? "auto";
    const apply = (dark: boolean) =>
      document.documentElement.classList.toggle("dark", dark);
    if (theme === "dark") {
      apply(true);
      return;
    }
    if (theme === "light") {
      apply(false);
      return;
    }
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    apply(media.matches);
    const onChange = (event: MediaQueryListEvent) => apply(event.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [settings?.theme]);

  const torOn = status?.socks_up ?? false;
  const proxyOn = status?.proxy.enabled ?? false;
  const tunOn = status?.tun.running ?? false;

  useEffect(() => {
    void refreshSession();
  }, [refreshSession, torOn]);

  const protectionLabel = useMemo(() => {
    if (!torOn) return "Disconnected";
    if (tunOn) return "Protected · TUN";
    if (proxyOn) {
      return settings?.bridges_enabled ? "Protected · bridges" : "Protected";
    }
    return settings?.bridges_enabled ? "Tor on · proxy off" : "Tor ready";
  }, [torOn, tunOn, proxyOn, settings?.bridges_enabled]);

  const toggleTor = () => {
    if (busy || !status?.tor_installed) return;
    void run(
      async () =>
        torOn ? invoke<string>("stop_tor") : invoke<string>("start_tor"),
      { refreshIps: !torOn },
    );
  };

  const toggleProxy = () => {
    if (busy || !status?.proxy.supported || !torOn) return;
    void run(
      () =>
        proxyOn
          ? invoke<string>("disable_proxy")
          : invoke<string>("enable_proxy"),
      { refreshIps: !proxyOn },
    );
  };

  const saveSettings = (patch: Partial<AppSettings>) => {
    if (!settings) return;
    const next = { ...settings, ...patch };
    setSettings(next);
    void run(async () => {
      const saved = await invoke<AppSettings>("update_settings", { next });
      setSettings(saved);
      setBridgeText(saved.bridge_lines.join("\n"));
      setExitDraft(saved.exit_country);
      if (patch.remote_dns !== undefined) {
        await invoke("set_remote_dns", { enabled: saved.remote_dns });
      }
      return "Settings saved";
    });
  };

  const newIdentity = () =>
    void run(async () => {
      const result = await invoke<NewIdentityResult>("new_identity");
      setIps(result.ips);
      return result.message;
    });

  // Changing the exit country pins ExitNodes and rotates the circuit (NEWNYM),
  // so refresh the IP once the new circuit settles to reflect the new location.
  const applyExitCountry = (country: string) =>
    void run(async () => {
      const result = await invoke<NewIdentityResult>("set_exit_country", {
        country,
      });
      setIps(result.ips);
      await refreshSettings();
      return result.message;
    });

  const fetchBridges = (transport?: string) =>
    void run(async () => {
      const t = transport ?? scanTransport;
      const res = await invoke<FetchBridgesResult>("fetch_bridges_for", {
        transport: t,
      });
      setCatalogBridges(res.lines);
      setScanResults(null);
      const cache = res.from_cache ? " (cached)" : "";
      return `Catalog ${t}: ${res.lines.length} from ${res.source}${cache}`;
    });

  const saveBridges = () =>
    void run(async () => {
      const saved = await invoke<AppSettings>("set_bridge_lines", {
        text: bridgeText,
      });
      setSettings(saved);
      setBridgeText(saved.bridge_lines.join("\n"));
      return `Saved ${saved.bridge_lines.length} bridge line(s)`;
    });

  const scanBridges = (lines?: string[]) =>
    void run(async () => {
      const list =
        lines ??
        (catalogBridges.length
          ? catalogBridges
          : (settings?.bridge_lines ?? []));
      const results = await invoke<BridgeScanResult[]>("scan_bridges", {
        lines: list,
      });
      setScanResults(results);
      const ok = results.filter((r) => r.ok).length;
      return `Scanned ${ok}/${results.length} reachable`;
    });

  const applyReachableBridges = () =>
    void run(async () => {
      const okLines = scanResults?.filter((r) => r.ok).map((r) => r.raw) ?? [];
      const msg = await invoke<string>("apply_scanned_bridges", {
        lines: okLines,
        enable: true,
      });
      await refreshSettings();
      if (okLines.length) {
        await invoke("save_bridge_library", { lines: okLines });
      }
      return msg;
    });

  const searchRelays = () =>
    void run(async () => {
      const list = await invoke<RelayInfo[]>("search_relays", {
        query: relayQuery,
        limit: 15,
      });
      setRelays(list);
      return `Found ${list.length} relay(s)`;
    });

  return {
    tab,
    setTab,
    systemView,
    setSystemView,
    status,
    ips,
    detect,
    shellProxy,
    networkTest,
    setNetworkTest,
    settings,
    logs,
    bridgeText,
    setBridgeText,
    catalogBridges,
    scanResults,
    exitDraft,
    setExitDraft,
    exitCountries,
    relayQuery,
    setRelayQuery,
    relays,
    snowflake,
    harden,
    session,
    scanTransport,
    setScanTransport,
    busy,
    message,
    error,
    bootstrapError,
    retryBootstrap: bootstrap,
    clearFlash,
    torOn,
    proxyOn,
    tunOn,
    protectionLabel,
    run,
    refreshIps,
    refreshSettings,
    refreshSnowflake,
    refreshHarden,
    refreshLogs,
    refreshDetect,
    refreshShellProxy,
    refreshSession,
    toggleTor,
    toggleProxy,
    saveSettings,
    newIdentity,
    applyExitCountry,
    fetchBridges,
    saveBridges,
    scanBridges,
    applyReachableBridges,
    searchRelays,
  };
}

export type TorApp = ReturnType<typeof useTorApp>;

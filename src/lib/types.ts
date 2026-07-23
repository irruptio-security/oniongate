export type Tab = "home" | "apps" | "onion-lab" | "verify" | "system";

/** Sub-views within the System tab (used for deep-linking from other pages). */
export type SystemView =
  | "audit"
  | "persistence"
  | "artifact"
  | "harden"
  | "settings"
  | "logs";

export type ProxyStatus = {
  supported: boolean;
  enabled: boolean;
  detail: string;
  host: string;
  port: number;
};

export type PtStatus = {
  transport: string;
  binary: string | null;
  available: boolean;
};

export type TunStatus = {
  supported: boolean;
  running: boolean;
  singbox_available: boolean;
  singbox_path: string | null;
  config_path: string | null;
  detail: string;
};

export type FirewallStatus = {
  supported: boolean;
  active: boolean;
  verified_live: boolean;
  marker_active: boolean;
  detail: string;
};

export type DepStatus = {
  name: string;
  path: string | null;
  available: boolean;
  hint: string;
};

export type AppStatus = {
  tor_installed: boolean;
  tor_path: string | null;
  socks_up: boolean;
  control_up: boolean;
  dns_up: boolean;
  remote_dns: boolean;
  bridges_enabled: boolean;
  bridge_count: number;
  smart_connect: boolean;
  exit_country: string;
  bootstrap_progress: number | null;
  connection_mode: string;
  kill_switch: boolean;
  pt: PtStatus[];
  tun: TunStatus;
  firewall: FirewallStatus;
  deps: DepStatus[];
  proxy: ProxyStatus;
  socks_host: string;
  socks_port: number;
  control_port: number;
  dns_port: number;
  install_hint: string;
  persistence_changes: number;
};

export type GeoLocation = {
  label: string;
  city: string | null;
  region: string | null;
  country: string | null;
  country_code: string | null;
};

export type IpReport = {
  direct_ip: string | null;
  tor_ip: string | null;
  direct_location: GeoLocation | null;
  tor_location: GeoLocation | null;
  direct_error: string | null;
  tor_error: string | null;
};

export type AdvancedItem = {
  id: string;
  group: string;
  title: string;
  description: string;
  configured: boolean;
  detail: string;
  configure_label: string;
  can_remove: boolean;
  note: string | null;
};

export type AdvancedStatus = {
  items: AdvancedItem[];
};

export type DetectedApp = {
  id: string;
  title: string;
  group: string;
  description: string;
  installed: boolean;
  configured: boolean;
  detail: string;
  note: string;
  configure_label: string;
  can_remove: boolean;
  process_names: string[];
  os: string;
};

export type DetectReport = {
  os: string;
  os_label: string;
  apps: DetectedApp[];
};

export type ExitCountryOption = {
  code: string;
  label: string;
};

export type SplitAppPick = {
  process_name: string;
  label: string;
  path: string;
  id: string;
  executable_path: string;
  bundle_id: string | null;
  signing_id: string | null;
};

export type NewIdentityResult = {
  message: string;
  ips: IpReport;
};

export type AppSettings = {
  remote_dns: boolean;
  auto_enable_proxy: boolean;
  auto_disable_proxy: boolean;
  log_level: string;
  status_poll_secs: number;
  locale: "auto" | "en" | "ru" | "fa" | "zh-CN" | "tr";
  theme: "auto" | "light" | "dark";
  smart_connect: boolean;
  bridges_enabled: boolean;
  bridge_lines: string[];
  exit_country: string;
  last_connect_strategy: string;
  last_network_key: string;
  last_connect_reason: string;
  bridge_source: string;
  connection_mode: string;
  kill_switch: boolean;
  split_tunnel: boolean;
  split_tunnel_apps: string[];
  route_apps: AppIdentity[];
  app_routing_policy: "only" | "except";
  session_guard: boolean;
  circuit_epoch: number;
  entry_nodes: string;
  middle_nodes: string;
  exit_nodes_fp: string;
  setup_complete: boolean;
};

export type AppIdentity = {
  id: string;
  label: string;
  process_name: string;
  executable_path: string;
  bundle_id: string | null;
  signing_id: string | null;
  circuit_epoch: number;
};

export type BridgeScanResult = {
  raw: string;
  transport: string;
  endpoint: string | null;
  ok: boolean;
  latency_ms: number | null;
  error: string | null;
};

export type FetchBridgesResult = {
  lines: string[];
  source: string;
  transport: string;
  from_cache: boolean;
};

export type SessionOverview = {
  bytes_read: number;
  bytes_written: number;
  bytes_total: number;
  circuits: number;
  identity_changes: number;
  rate_down_bps: number;
  rate_up_bps: number;
  uptime_secs: number;
  started_at: number | null;
  connected: boolean;
};

export type RelayInfo = {
  nickname: string;
  fingerprint: string;
  country: string | null;
  as_name: string | null;
  flags: string[];
  or_addresses: string[];
  observed_bandwidth: number | null;
};

export type SnowflakeStatus = {
  binary: string | null;
  available: boolean;
  running: boolean;
  detail: string;
};

export type HardenItem = {
  id: string;
  title: string;
  description: string;
  active: boolean;
  supported: boolean;
  detail: string;
  group: string;
  control: string;
  risk: string;
};

export type KillSiriStatus = {
  installed: boolean;
  agent_loaded: boolean;
  running: string[];
  total_watched: number;
  detail: string;
};

export type MacPortsStatus = {
  installed: boolean;
  version: string;
  path: string;
  macos_version: string;
  macos_name: string;
  download_url: string;
  install_page: string;
  detail: string;
};

export type TorLogs = {
  lines: string[];
  source: string;
  log_path: string | null;
};

export type ShellProxyStatus = {
  mode: "off" | "auto" | "manual" | string;
  script: string;
  script_path: string;
  detail: string;
};

export type NetworkTestResult = {
  success: boolean;
  message: string;
  direct_ip: string | null;
  tor_ip: string | null;
};

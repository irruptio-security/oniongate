import type { AppSettings } from "@/lib/types";

type Locale = Exclude<AppSettings["locale"], "auto">;
type Key =
  | "connect"
  | "subtitle"
  | "working"
  | "connectAction"
  | "disconnectAction"
  | "connectHint"
  | "disconnectHint"
  | "interrupted"
  | "restore";

const EN: Record<Key, string> = {
  connect: "Connect",
  subtitle: "Route apps through Tor. Bridges stay off unless the network blocks Tor.",
  working: "Working…",
  connectAction: "Connect Tor",
  disconnectAction: "Disconnect Tor",
  connectHint: "Click to connect",
  disconnectHint: "Click to disconnect",
  interrupted: "Interrupted session detected",
  restore: "Emergency Restore",
};

const TEXT: Record<Locale, Record<Key, string>> = {
  en: EN,
  ru: {
    connect: "Подключение",
    subtitle: "Маршрутизация приложений через Tor. Мосты включаются только при блокировке Tor.",
    working: "Выполняется…",
    connectAction: "Подключить Tor",
    disconnectAction: "Отключить Tor",
    connectHint: "Нажмите для подключения",
    disconnectHint: "Нажмите для отключения",
    interrupted: "Обнаружен прерванный сеанс",
    restore: "Аварийное восстановление",
  },
  fa: {
    connect: "اتصال",
    subtitle: "برنامه‌ها را از Tor عبور دهید. پل‌ها فقط هنگام مسدود بودن Tor فعال می‌شوند.",
    working: "در حال انجام…",
    connectAction: "اتصال Tor",
    disconnectAction: "قطع Tor",
    connectHint: "برای اتصال کلیک کنید",
    disconnectHint: "برای قطع کلیک کنید",
    interrupted: "نشست ناتمام شناسایی شد",
    restore: "بازیابی اضطراری",
  },
  "zh-CN": {
    connect: "连接",
    subtitle: "通过 Tor 路由应用。仅在网络封锁 Tor 时启用网桥。",
    working: "正在处理…",
    connectAction: "连接 Tor",
    disconnectAction: "断开 Tor",
    connectHint: "点击连接",
    disconnectHint: "点击断开",
    interrupted: "检测到中断的会话",
    restore: "紧急恢复",
  },
  tr: {
    connect: "Bağlan",
    subtitle: "Uygulamaları Tor üzerinden yönlendir. Köprüleri yalnızca Tor engelliyse kullan.",
    working: "Çalışıyor…",
    connectAction: "Tor'a bağlan",
    disconnectAction: "Tor bağlantısını kes",
    connectHint: "Bağlanmak için tıklayın",
    disconnectHint: "Kesmek için tıklayın",
    interrupted: "Yarım kalan oturum algılandı",
    restore: "Acil Geri Yükleme",
  },
};

export function effectiveLocale(setting: AppSettings["locale"] | undefined): Locale {
  // Full localization is deferred: only English is complete, so "auto" resolves
  // to English instead of a partially-translated OS locale. Explicit non-English
  // selection is disabled in Settings until translations land, but is still
  // honored here so the scaffold keeps working when re-enabled.
  if (setting && setting !== "auto") return setting;
  return "en";
}

export function translate(locale: Locale, key: Key): string {
  return TEXT[locale]?.[key] ?? EN[key];
}

type IconProps = { className?: string };

export function PersonIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <circle cx="12" cy="8" r="3.25" stroke="currentColor" strokeWidth="1.8" />
      <path
        d="M5.5 19.5c1.4-3.2 3.7-4.8 6.5-4.8s5.1 1.6 6.5 4.8"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function RefreshIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        d="M19.5 12a7.5 7.5 0 1 1-2.1-5.2"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
      <path
        d="M19.5 5.5v4.2h-4.2"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function GearIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        d="M12 8.4a3.6 3.6 0 1 0 0 7.2 3.6 3.6 0 0 0 0-7.2z"
        stroke="currentColor"
        strokeWidth="1.7"
      />
      <path
        d="M19.4 13.2v-2.4l-1.7-.3a5.9 5.9 0 0 0-.5-1.1l1-1.4-1.7-1.7-1.4 1a5.9 5.9 0 0 0-1.1-.5L13.2 4.6h-2.4l-.3 1.7a5.9 5.9 0 0 0-1.1.5l-1.4-1-1.7 1.7 1 1.4a5.9 5.9 0 0 0-.5 1.1l-1.7.3v2.4l1.7.3c.1.4.3.8.5 1.1l-1 1.4 1.7 1.7 1.4-1c.3.2.7.4 1.1.5l.3 1.7h2.4l.3-1.7c.4-.1.8-.3 1.1-.5l1.4 1 1.7-1.7-1-1.4c.2-.3.4-.7.5-1.1l1.7-.3z"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function TerminalIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <rect
        x="3.5"
        y="4.5"
        width="17"
        height="15"
        rx="2"
        stroke="currentColor"
        strokeWidth="1.7"
      />
      <path
        d="M7 9.5 10 12l-3 2.5M12.5 14.5H17"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function BridgeIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        d="M4 16V9.5A3.5 3.5 0 0 1 7.5 6H9"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
      />
      <path
        d="M20 16V9.5A3.5 3.5 0 0 0 16.5 6H15"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
      />
      <path
        d="M4 16h16M7 16v3M12 16v3M17 16v3"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
      />
      <path
        d="M9 6h6"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function SlidersIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        d="M4 7h10M18 7h2M4 17h2M10 17h10M14 4v6M8 14v6"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function HomeIcon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        d="M4.5 11 12 4.5 19.5 11v8a1.5 1.5 0 0 1-1.5 1.5h-4.5v-5h-3v5H6A1.5 1.5 0 0 1 4.5 19v-8z"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinejoin="round"
      />
    </svg>
  );
}

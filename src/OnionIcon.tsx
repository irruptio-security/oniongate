type Props = {
  className?: string;
};

/** OnionGate mark: an onion standing within an arched gateway. Uses
 *  currentColor so it inherits state/theme colors (sidebar, connect orb). */
export function OnionIcon({ className }: Props) {
  return (
    <svg
      className={className}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
    >
      {/* Gate arch + posts */}
      <path
        d="M13 55V31a19 19 0 0 1 38 0v24"
        stroke="currentColor"
        strokeWidth="2.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      {/* Ground line */}
      <path
        d="M8 55h48"
        stroke="currentColor"
        strokeWidth="2.6"
        strokeLinecap="round"
      />
      {/* Onion bulb */}
      <path
        d="M32 24c-5.6 0-9.6 5.1-9.6 12 0 6.6 4.2 11.2 9.6 11.2s9.6-4.6 9.6-11.2c0-6.9-4-12-9.6-12z"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinejoin="round"
      />
      {/* Sprout */}
      <path
        d="M32 24c0-3.6 1.7-6.1 4.6-7.6"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
      {/* Inner layer hint */}
      <path
        d="M32 30.5c-2.4 0-4.1 2.5-4.1 6s1.7 6 4.1 6"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        opacity="0.55"
      />
    </svg>
  );
}

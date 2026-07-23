import { cn } from "@/lib/utils";

export function Segmented<T extends string>({
  value,
  options,
  onChange,
  disabled,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
}) {
  return (
    <div
      role="group"
      className="inline-flex rounded-lg border border-line bg-canvas p-0.5"
    >
      {options.map((opt) => (
        <button
          key={opt.value}
          type="button"
          disabled={disabled}
          aria-pressed={value === opt.value}
          onClick={() => onChange(opt.value)}
          className={cn(
            "rounded-md px-3 py-1.5 text-xs font-semibold transition-colors disabled:opacity-45",
            value === opt.value
              ? "bg-panel-2 text-ink shadow-sm"
              : "text-muted hover:text-ink",
          )}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}

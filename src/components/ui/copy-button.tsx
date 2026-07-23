import { useState } from "react";
import { Button, type ButtonProps } from "@/components/ui/button";

/**
 * Copies text to the clipboard and shows brief inline "Copied" feedback, so
 * copy actions are not silent. Falls back to the given label after ~1.5s.
 */
export function CopyButton({
  value,
  label = "Copy",
  copiedLabel = "Copied",
  size = "sm",
  variant = "ghost",
  className,
  disabled,
}: {
  value: string;
  label?: string;
  copiedLabel?: string;
  size?: ButtonProps["size"];
  variant?: ButtonProps["variant"];
  className?: string;
  disabled?: boolean;
}) {
  const [copied, setCopied] = useState(false);
  return (
    <Button
      size={size}
      variant={variant}
      className={className}
      disabled={disabled}
      onClick={() => {
        void navigator.clipboard.writeText(value).then(() => {
          setCopied(true);
          window.setTimeout(() => setCopied(false), 1500);
        });
      }}
    >
      {copied ? copiedLabel : label}
    </Button>
  );
}

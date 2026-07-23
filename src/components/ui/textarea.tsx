import * as React from "react";
import { cn } from "@/lib/utils";

export const Textarea = React.forwardRef<
  HTMLTextAreaElement,
  React.TextareaHTMLAttributes<HTMLTextAreaElement>
>(({ className, ...props }, ref) => (
  <textarea
    className={cn(
      "flex min-h-[120px] w-full rounded-lg border border-line bg-canvas px-3 py-2 font-mono text-xs leading-relaxed text-ink placeholder:text-muted/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 disabled:opacity-45",
      className,
    )}
    ref={ref}
    {...props}
  />
));
Textarea.displayName = "Textarea";

import * as React from "react";
import { cn } from "@/lib/utils";

export const Input = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, type, ...props }, ref) => (
  <input
    type={type}
    className={cn(
      "flex h-9 w-full rounded-lg border border-line bg-canvas px-3 text-sm text-ink placeholder:text-muted/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 disabled:opacity-45",
      className,
    )}
    ref={ref}
    {...props}
  />
));
Input.displayName = "Input";

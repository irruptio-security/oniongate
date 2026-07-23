import * as React from "react";
import { cn } from "@/lib/utils";

export const Select = React.forwardRef<
  HTMLSelectElement,
  React.SelectHTMLAttributes<HTMLSelectElement>
>(({ className, children, ...props }, ref) => (
  <select
    className={cn(
      "flex h-9 w-full appearance-none rounded-lg border border-line bg-canvas px-3 pr-8 text-sm text-ink focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 disabled:opacity-45",
      "bg-[length:12px_12px] bg-[right_0.65rem_center] bg-no-repeat",
      "bg-[url('data:image/svg+xml;charset=utf-8,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%2212%22 height=%2212%22 viewBox=%220 0 24 24%22 fill=%22none%22 stroke=%22%236b7280%22 stroke-width=%222%22%3E%3Cpath d=%22m6 9 6 6 6-6%22/%3E%3C/svg%3E')]",
      className,
    )}
    ref={ref}
    {...props}
  >
    {children}
  </select>
));
Select.displayName = "Select";

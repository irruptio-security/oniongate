import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-md px-2 py-0.5 text-[11px] font-semibold tracking-wide uppercase",
  {
    variants: {
      variant: {
        default: "bg-panel-2 text-muted border border-line",
        success: "bg-accent/15 text-accent-strong border border-accent/30",
        warn: "bg-warn/15 text-warn-strong border border-warn/30",
        danger: "bg-danger/15 text-danger-strong border border-danger/30",
        onion: "bg-onion/15 text-onion-strong border border-onion/30",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

export function Badge({
  className,
  variant,
  ...props
}: React.HTMLAttributes<HTMLSpanElement> &
  VariantProps<typeof badgeVariants>) {
  return (
    <span className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

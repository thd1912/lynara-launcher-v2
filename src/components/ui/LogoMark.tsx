import { motion } from "framer-motion";
import { cn } from "../../lib/utils";

interface LogoMarkProps {
  size?: number;
  glow?: boolean | "strong";
  floating?: boolean;
  pulseRing?: boolean;
  className?: string;
}

/**
 * LogoMark - le logo Lynara avec options de glow, flottement, anneau pulsant.
 * Le PNG est dans public/lynara-logo.png
 */
export default function LogoMark({
  size = 64,
  glow = false,
  floating = false,
  pulseRing = false,
  className,
}: LogoMarkProps) {
  return (
    <div
      className={cn("relative inline-block", className)}
      style={{ width: size, height: size }}
    >
      {/* Pulsing ring (optional) */}
      {pulseRing && (
        <>
          <div
            className="absolute inset-0 rounded-full bg-accent/15 animate-[pulse-ring_3s_ease-in-out_infinite]"
            style={{ animationDelay: "0s" }}
          />
          <div
            className="absolute inset-0 rounded-full bg-accent/10 animate-[pulse-ring_3s_ease-in-out_infinite]"
            style={{ animationDelay: "1.5s" }}
          />
        </>
      )}

      {/* Logo with optional float animation */}
      <motion.img
        src="/lynara-logo.png"
        alt="Lynara"
        width={size}
        height={size}
        className={cn(
          "relative w-full h-full object-contain select-none pointer-events-none",
          glow === "strong" && "logo-glow-strong",
          glow === true && "logo-glow"
        )}
        style={{
          animation: floating ? "float-y 4.5s ease-in-out infinite" : undefined,
        }}
        draggable={false}
      />
    </div>
  );
}

/**
 * AnimatedBackground
 * 3 layers : base gradient + drifting blobs + starfield + vignette
 * Pure CSS, GPU-accelerated, no perf cost
 */
export default function AnimatedBackground() {
  return (
    <div className="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      {/* Base gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-bg-deep via-bg to-[#0c0617]" />

      {/* Drifting blobs (gradient orbs) */}
      <div className="absolute top-[-15%] left-[-10%] w-[600px] h-[600px] rounded-full bg-accent/[0.10] blur-[120px] blob-1" />
      <div className="absolute top-[20%] right-[-15%] w-[700px] h-[700px] rounded-full bg-[#8c50b4]/[0.10] blur-[140px] blob-2" />
      <div className="absolute bottom-[-20%] left-[25%] w-[500px] h-[500px] rounded-full bg-accent/[0.06] blur-[110px] blob-3" />

      {/* Star field */}
      <div className="absolute inset-0 stars-field" />

      {/* Subtle noise grain */}
      <div
        className="absolute inset-0 opacity-[0.018]"
        style={{
          backgroundImage:
            "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='2' stitchTiles='stitch'/></filter><rect width='160' height='160' filter='url(%23n)' opacity='0.5'/></svg>\")",
        }}
      />

      {/* Top accent glow */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-2/3 h-px bg-gradient-to-r from-transparent via-accent/40 to-transparent" />

      {/* Vignette */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_30%,rgba(6,2,10,0.7)_100%)]" />
    </div>
  );
}

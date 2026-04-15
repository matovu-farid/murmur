import { useEffect, useRef } from "react";

export function Waveform({ isActive }: { isActive: boolean }) {
  const barsRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!isActive || !barsRef.current) return;
    const bars = barsRef.current.children;
    const interval = setInterval(() => {
      for (let i = 0; i < bars.length; i++) {
        (bars[i] as HTMLElement).style.height = `${Math.random() * 100}%`;
      }
    }, 100);
    return () => clearInterval(interval);
  }, [isActive]);
  return (
    <div ref={barsRef} className="flex items-end gap-0.5 h-8">
      {Array.from({ length: 12 }).map((_, i) => (
        <div
          key={i}
          className="w-1 bg-primary rounded-full transition-all duration-100"
          style={{ height: "10%" }}
        />
      ))}
    </div>
  );
}

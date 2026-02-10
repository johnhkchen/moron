import type { ReactNode, CSSProperties } from "react";

export interface MetricProps {
  /** The numeric or text value to display prominently. */
  value: ReactNode;
  /** A short label describing the metric. */
  label?: ReactNode;
  /** Optional unit suffix (e.g. "%", "ms", "users"). */
  unit?: string;
  className?: string;
  style?: CSSProperties;
}

/**
 * Stat / KPI display component for a Moron scene.
 * Shows a prominent value with an optional label and unit.
 */
export function Metric({ value, label, unit, className, style }: MetricProps) {
  return (
    <div data-moron="metric" className={className} style={style}>
      <span data-moron="metric-value">
        {value}
        {unit && <span data-moron="metric-unit">{unit}</span>}
      </span>
      {label && <span data-moron="metric-label">{label}</span>}
    </div>
  );
}

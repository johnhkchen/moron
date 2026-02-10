import { Children, type ReactNode, type CSSProperties } from "react";

export interface SequenceProps {
  children?: ReactNode;
  className?: string;
  style?: CSSProperties;
  direction?: "horizontal" | "vertical";
  gap?: string | number;
}

/**
 * Ordered sequence of items within a Moron scene.
 * Renders each child as a discrete item that can be individually animated.
 */
export function Sequence({
  children,
  className,
  style,
  direction = "vertical",
  gap = 0,
}: SequenceProps) {
  return (
    <div
      data-moron="sequence"
      className={className}
      style={{
        display: "flex",
        flexDirection: direction === "horizontal" ? "row" : "column",
        gap,
        ...style,
      }}
    >
      {Children.map(children, (child, index) => (
        <div data-moron="sequence-item" data-index={index}>
          {child}
        </div>
      ))}
    </div>
  );
}

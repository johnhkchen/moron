import type { ReactNode, CSSProperties } from "react";

export interface ContainerProps {
  children?: ReactNode;
  className?: string;
  style?: CSSProperties;
}

/**
 * Root layout container for a Moron scene.
 * Wraps content in a full-viewport frame that the renderer captures.
 */
export function Container({ children, className, style }: ContainerProps) {
  return (
    <div
      data-moron="container"
      className={className}
      style={{
        width: "100%",
        height: "100%",
        position: "relative",
        overflow: "hidden",
        ...style,
      }}
    >
      {children}
    </div>
  );
}

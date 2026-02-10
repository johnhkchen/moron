import type { ReactNode, CSSProperties } from "react";

export interface TitleProps {
  children?: ReactNode;
  className?: string;
  style?: CSSProperties;
  level?: 1 | 2 | 3;
}

/**
 * Title text element for a Moron scene.
 * Renders a heading whose level defaults to 1.
 */
export function Title({ children, className, style, level = 1 }: TitleProps) {
  const Tag = `h${level}` as const;

  return (
    <Tag
      data-moron="title"
      className={className}
      style={style}
    >
      {children}
    </Tag>
  );
}

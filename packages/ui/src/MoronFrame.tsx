import type { CSSProperties, ReactNode } from "react";

import type { ElementState, FrameState, ItemState } from "./types";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface MoronFrameProps {
  /** The complete visual state for this frame. */
  state: FrameState;
  /** Frame width in pixels. Defaults to 1920. */
  width?: number;
  /** Frame height in pixels. Defaults to 1080. */
  height?: number;
}

// ---------------------------------------------------------------------------
// Transform builder
// ---------------------------------------------------------------------------

/**
 * Build a CSS `transform` string from an element's visual state.
 * Opacity is handled separately as a CSS property, not inside transform.
 */
function buildTransform(el: ElementState): string {
  const parts: string[] = [];

  if (el.translateX !== 0 || el.translateY !== 0) {
    parts.push(`translate(${el.translateX}px, ${el.translateY}px)`);
  }
  if (el.scale !== 1) {
    parts.push(`scale(${el.scale})`);
  }
  if (el.rotation !== 0) {
    parts.push(`rotate(${el.rotation}deg)`);
  }

  return parts.length > 0 ? parts.join(" ") : "none";
}

/**
 * Build a CSS `transform` string from an item's per-item visual state.
 */
function buildItemTransform(item: ItemState): string {
  const parts: string[] = [];

  if (item.translateX !== 0 || item.translateY !== 0) {
    parts.push(`translate(${item.translateX}px, ${item.translateY}px)`);
  }
  if (item.scale !== 1) {
    parts.push(`scale(${item.scale})`);
  }
  if (item.rotation !== 0) {
    parts.push(`rotate(${item.rotation}deg)`);
  }

  return parts.length > 0 ? parts.join(" ") : "none";
}

// ---------------------------------------------------------------------------
// Content renderers
// ---------------------------------------------------------------------------

/**
 * Render the inner content of an element based on its kind.
 * Uses var(--moron-*) CSS custom properties for all styling.
 */
function renderContent(el: ElementState): ReactNode {
  const kind = el.kind;

  switch (kind.type) {
    case "title":
      return (
        <h1
          data-moron="title"
          style={{
            margin: 0,
            padding: 0,
            fontSize: "var(--moron-text-4xl)",
            fontWeight: "var(--moron-font-weight-bold)" as CSSProperties["fontWeight"],
            lineHeight: 1.2,
            textAlign: "center" as const,
          }}
        >
          {el.content}
        </h1>
      );

    case "section":
      return (
        <h2
          data-moron="title"
          style={{
            margin: 0,
            padding: 0,
            fontSize: "var(--moron-text-2xl)",
            fontWeight: "var(--moron-font-weight-semibold)" as CSSProperties["fontWeight"],
            lineHeight: 1.3,
            textAlign: "center" as const,
          }}
        >
          {el.content}
        </h2>
      );

    case "show":
      return (
        <p
          data-moron="show"
          style={{
            margin: 0,
            padding: 0,
            fontSize: "var(--moron-text-xl)",
            lineHeight: 1.5,
            textAlign: "center" as const,
          }}
        >
          {el.content}
        </p>
      );

    case "metric": {
      // Rust facade stores metric content as "label: value" (e.g. "Revenue: $1M").
      // Split on first colon to extract label and value parts.
      const colonIndex = el.content.indexOf(": ");
      const label = colonIndex >= 0 ? el.content.slice(0, colonIndex) : "";
      const value = colonIndex >= 0 ? el.content.slice(colonIndex + 2) : el.content;

      return (
        <div
          data-moron="metric"
          data-direction={kind.direction}
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: "0.5em",
          }}
        >
          <span
            data-moron="metric-value"
            style={{
              fontSize: "var(--moron-text-4xl)",
              fontWeight: "var(--moron-font-weight-bold)" as CSSProperties["fontWeight"],
              lineHeight: 1.2,
            }}
          >
            {value}
          </span>
          {label && (
            <span
              data-moron="metric-label"
              style={{
                fontSize: "var(--moron-text-lg)",
                opacity: 0.8,
              }}
            >
              {label}
            </span>
          )}
        </div>
      );
    }

    case "steps": {
      return (
        <div
          data-moron="sequence"
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "var(--moron-space-4)",
            alignItems: "flex-start",
          }}
        >
          {el.items.map((item, i) => {
            const itemTransform = buildItemTransform(item);
            const itemStyle: CSSProperties = {
              fontSize: "var(--moron-text-xl)",
              lineHeight: 1.5,
              opacity: item.opacity,
              transform: itemTransform !== "none" ? itemTransform : undefined,
            };
            return (
              <div
                key={i}
                data-moron="sequence-item"
                data-index={i}
                style={itemStyle}
              >
                {item.text}
              </div>
            );
          })}
        </div>
      );
    }
  }
}

// ---------------------------------------------------------------------------
// MoronFrame component
// ---------------------------------------------------------------------------

/**
 * Renders a single frame from serialized FrameState JSON.
 *
 * This is a pure component: same FrameState input always produces the same
 * visual output. No animation, no state management, no side effects.
 *
 * Theme CSS custom properties are applied as inline styles on the root
 * container, cascading to all children via CSS variable inheritance.
 *
 * Z-ordering follows element array order: index 0 is lowest, last is highest.
 */
export function MoronFrame({
  state,
  width = 1920,
  height = 1080,
}: MoronFrameProps) {
  // Build theme CSS custom properties as inline style entries.
  const themeStyles: Record<string, string> = {};
  for (const [key, value] of Object.entries(state.theme.cssProperties)) {
    themeStyles[key] = value;
  }

  const containerStyle: CSSProperties = {
    width: `${width}px`,
    height: `${height}px`,
    position: "relative",
    overflow: "hidden",
    background: "var(--moron-bg-primary)",
    color: "var(--moron-fg-primary)",
    fontFamily: "var(--moron-font-sans)",
    ...themeStyles,
  };

  return (
    <div data-moron="frame" style={containerStyle}>
      {state.elements.map((el, index) => {
        // Skip invisible elements entirely.
        if (!el.visible) {
          return null;
        }

        // Position element using layout_y (0=top, 0.5=center, 1=bottom).
        // translate(-50%, -50%) centers the element on its anchor point.
        // Animation transforms (translate, scale, rotate) compose after.
        const animTransform = buildTransform(el);
        const centerAndAnim = animTransform !== "none"
          ? `translate(-50%, -50%) ${animTransform}`
          : "translate(-50%, -50%)";

        const wrapperStyle: CSSProperties = {
          position: "absolute",
          top: `${el.layoutY * 100}%`,
          left: "50%",
          transform: centerAndAnim,
          maxWidth: "80%",
          opacity: el.opacity,
          zIndex: index,
          pointerEvents: "none",
          textAlign: "center",
        };

        return (
          <div
            key={el.id}
            data-moron="element"
            data-element-id={el.id}
            style={wrapperStyle}
          >
            {renderContent(el)}
          </div>
        );
      })}
    </div>
  );
}

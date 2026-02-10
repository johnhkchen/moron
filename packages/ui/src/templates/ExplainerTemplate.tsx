/**
 * Explainer template for Moron scenes.
 *
 * A production-quality template that renders all 5 element types (title,
 * section, show, metric, steps) with polished visual styling. All colors
 * and typography reference var(--moron-*) CSS custom properties from the
 * theme system.
 *
 * Self-registers as "explainer" in the template registry on module load.
 */

import type { CSSProperties, ReactNode } from "react";

import type { ElementState, FrameState } from "../types";
import { registerTemplate } from "./registry";
import type { TemplateProps } from "./registry";

// ---------------------------------------------------------------------------
// Transform builder (duplicated from MoronFrame — extraction is out of scope)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Sub-components (internal — not exported)
// ---------------------------------------------------------------------------

/**
 * Title card: large centered text with accent underline and radial gradient.
 */
function ExplainerTitle({ el }: { el: ElementState }): ReactNode {
  return (
    <div
      data-moron="explainer-title"
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "var(--moron-space-6)",
        padding: "var(--moron-container-padding)",
        background:
          "radial-gradient(ellipse at center, var(--moron-accent-subtle) 0%, transparent 70%)",
      }}
    >
      <h1
        style={{
          margin: 0,
          padding: 0,
          fontSize: "var(--moron-text-4xl)",
          fontWeight: "var(--moron-font-weight-bold)" as CSSProperties["fontWeight"],
          lineHeight: "var(--moron-leading-tight)",
          textAlign: "center" as const,
          color: "var(--moron-fg-primary)",
        }}
      >
        {el.content}
      </h1>
      {/* Accent underline bar */}
      <div
        style={{
          width: "80px",
          height: "4px",
          backgroundColor: "var(--moron-accent)",
          borderRadius: "var(--moron-radius-full)",
        }}
      />
    </div>
  );
}

/**
 * Section heading: centered text with accent left border.
 */
function ExplainerSection({ el }: { el: ElementState }): ReactNode {
  return (
    <div
      data-moron="explainer-section"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(--moron-container-padding)",
      }}
    >
      <h2
        style={{
          margin: 0,
          padding: 0,
          paddingLeft: "var(--moron-space-6)",
          borderLeft: "4px solid var(--moron-accent)",
          fontSize: "var(--moron-text-3xl)",
          fontWeight: "var(--moron-font-weight-semibold)" as CSSProperties["fontWeight"],
          lineHeight: "var(--moron-leading-tight)",
          color: "var(--moron-fg-primary)",
        }}
      >
        {el.content}
      </h2>
    </div>
  );
}

/**
 * Show text: readable body text with width constraint.
 */
function ExplainerShow({ el }: { el: ElementState }): ReactNode {
  return (
    <div
      data-moron="explainer-show"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "0 var(--moron-container-padding)",
      }}
    >
      <p
        style={{
          margin: 0,
          padding: 0,
          maxWidth: "75%",
          fontSize: "var(--moron-text-xl)",
          fontWeight: "var(--moron-font-weight-normal)" as CSSProperties["fontWeight"],
          lineHeight: "var(--moron-leading-normal)",
          textAlign: "center" as const,
          color: "var(--moron-fg-secondary)",
        }}
      >
        {el.content}
      </p>
    </div>
  );
}

/**
 * Metric display: card with value, direction arrow, and label.
 */
function ExplainerMetric({ el }: { el: ElementState }): ReactNode {
  // Parse "label: value" format (same convention as MoronFrame).
  const colonIndex = el.content.indexOf(": ");
  const label = colonIndex >= 0 ? el.content.slice(0, colonIndex) : "";
  const value = colonIndex >= 0 ? el.content.slice(colonIndex + 2) : el.content;

  // Direction indicator: color and arrow character.
  const direction = el.kind.type === "metric" ? el.kind.direction : "neutral";

  let directionColor: string;
  let directionArrow: string;
  switch (direction) {
    case "up":
      directionColor = "var(--moron-success)";
      directionArrow = "\u2191"; // U+2191 UPWARDS ARROW
      break;
    case "down":
      directionColor = "var(--moron-error)";
      directionArrow = "\u2193"; // U+2193 DOWNWARDS ARROW
      break;
    default:
      directionColor = "var(--moron-fg-muted)";
      directionArrow = "";
      break;
  }

  return (
    <div
      data-moron="explainer-metric"
      data-direction={direction}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(--moron-container-padding)",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "var(--moron-space-4)",
          background: "var(--moron-bg-secondary)",
          borderRadius: "var(--moron-radius-lg)",
          padding: "var(--moron-space-8) var(--moron-space-12)",
          boxShadow: "var(--moron-shadow-md)",
        }}
      >
        <span
          data-moron="explainer-metric-value"
          style={{
            fontSize: "var(--moron-text-4xl)",
            fontWeight: "var(--moron-font-weight-bold)" as CSSProperties["fontWeight"],
            lineHeight: "var(--moron-leading-tight)",
            color: "var(--moron-fg-primary)",
          }}
        >
          {directionArrow && (
            <span style={{ color: directionColor, marginRight: "var(--moron-space-2)" }}>
              {directionArrow}
            </span>
          )}
          {value}
        </span>
        {label && (
          <span
            data-moron="explainer-metric-label"
            style={{
              fontSize: "var(--moron-text-lg)",
              color: "var(--moron-fg-muted)",
            }}
          >
            {label}
          </span>
        )}
      </div>
    </div>
  );
}

/**
 * Steps list: numbered badges with step text.
 */
function ExplainerSteps({ el }: { el: ElementState }): ReactNode {
  return (
    <div
      data-moron="explainer-steps"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(--moron-container-padding)",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "var(--moron-space-6)",
          maxWidth: "75%",
          width: "100%",
        }}
      >
        {el.items.map((item, i) => (
          <div
            key={i}
            data-moron="explainer-step-item"
            data-index={i}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "var(--moron-space-4)",
            }}
          >
            {/* Numbered circular badge */}
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: "2em",
                height: "2em",
                minWidth: "2em",
                borderRadius: "var(--moron-radius-full)",
                backgroundColor: "var(--moron-accent)",
                color: "var(--moron-fg-primary)",
                fontSize: "var(--moron-text-base)",
                fontWeight: "var(--moron-font-weight-bold)" as CSSProperties["fontWeight"],
                lineHeight: 1,
              }}
            >
              {i + 1}
            </div>
            {/* Step text */}
            <span
              style={{
                fontSize: "var(--moron-text-xl)",
                lineHeight: "var(--moron-leading-normal)",
                color: "var(--moron-fg-primary)",
              }}
            >
              {item}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Content router
// ---------------------------------------------------------------------------

function renderExplainerContent(el: ElementState): ReactNode {
  switch (el.kind.type) {
    case "title":
      return <ExplainerTitle el={el} />;
    case "section":
      return <ExplainerSection el={el} />;
    case "show":
      return <ExplainerShow el={el} />;
    case "metric":
      return <ExplainerMetric el={el} />;
    case "steps":
      return <ExplainerSteps el={el} />;
  }
}

// ---------------------------------------------------------------------------
// ExplainerTemplate component
// ---------------------------------------------------------------------------

/**
 * Renders a polished explainer-style frame from FrameState.
 *
 * Supports all 5 element kinds with professional visual treatment.
 * All styling uses var(--moron-*) CSS custom properties — no hardcoded
 * colors. Responsive to frame dimensions via proportional root font-size.
 */
export function ExplainerTemplate({
  state,
  width = 1920,
  height = 1080,
}: TemplateProps) {
  // Responsive font sizing: scale proportionally to frame width.
  // At 1920px wide, 1rem = 16px (CSS default). Scales linearly.
  const baseFontSize = (width / 1920) * 16;

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
    fontSize: `${baseFontSize}px`,
    ...themeStyles,
  };

  return (
    <div data-moron="frame" data-template="explainer" style={containerStyle}>
      {state.elements.map((el, index) => {
        // Skip invisible elements entirely.
        if (!el.visible) {
          return null;
        }

        const wrapperStyle: CSSProperties = {
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          opacity: el.opacity,
          transform: buildTransform(el),
          zIndex: index,
          pointerEvents: "none",
        };

        return (
          <div
            key={el.id}
            data-moron="element"
            data-element-id={el.id}
            style={wrapperStyle}
          >
            {renderExplainerContent(el)}
          </div>
        );
      })}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Self-registration
// ---------------------------------------------------------------------------

registerTemplate("explainer", ExplainerTemplate);

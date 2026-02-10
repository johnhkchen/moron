/**
 * TypeScript types matching the Rust FrameState JSON contract.
 *
 * These types mirror the serde-serialized output of the Rust structs in
 * moron-core/src/frame.rs. Field names use camelCase to match Rust's
 * #[serde(rename_all = "camelCase")].
 *
 * Keep in sync with moron-core/src/frame.rs.
 */

// ---------------------------------------------------------------------------
// ElementKind — discriminated union matching Rust's #[serde(tag = "type")]
// ---------------------------------------------------------------------------

/**
 * The structural type of a visual element.
 *
 * Rust: `frame::ElementKind` with `#[serde(tag = "type")]`.
 * Variants without data serialize as `{ "type": "title" }`.
 * Variants with fields include them at the same level.
 */
export type ElementKind =
  | { type: "title" }
  | { type: "show" }
  | { type: "section" }
  | { type: "metric"; direction: string }
  | { type: "steps"; count: number };

// ---------------------------------------------------------------------------
// ElementState — per-element visual snapshot
// ---------------------------------------------------------------------------

/**
 * The visual state of a single element at a point in time.
 *
 * Rust: `frame::ElementState` with `#[serde(rename_all = "camelCase")]`.
 * Visual transform fields are flat (not nested under a sub-object).
 */
export interface ElementState {
  /** Unique element identifier (u64 in Rust). */
  id: number;
  /** Structural type of this element. */
  kind: ElementKind;
  /** Primary text content. */
  content: string;
  /** List items (non-empty only for Steps elements). */
  items: string[];
  /** Whether this element is currently visible. */
  visible: boolean;
  /** Opacity: 0.0 = transparent, 1.0 = fully opaque. */
  opacity: number;
  /** Horizontal translation in pixels. */
  translateX: number;
  /** Vertical translation in pixels. */
  translateY: number;
  /** Scale factor: 1.0 = normal size. */
  scale: number;
  /** Rotation in degrees. */
  rotation: number;
}

// ---------------------------------------------------------------------------
// ThemeState — theme as CSS custom properties
// ---------------------------------------------------------------------------

/**
 * Theme snapshot serialized as CSS custom property pairs.
 *
 * Rust: `frame::ThemeState` with `#[serde(rename_all = "camelCase")]`.
 * The `css_properties` field becomes `cssProperties`.
 */
export interface ThemeState {
  /** Theme name. */
  name: string;
  /** CSS custom properties: `--moron-*` key-value pairs. */
  cssProperties: Record<string, string>;
}

// ---------------------------------------------------------------------------
// FrameState — the complete visual state at a timestamp
// ---------------------------------------------------------------------------

/**
 * Complete visual state at a given point in the timeline.
 *
 * Rust: `frame::FrameState` with `#[serde(rename_all = "camelCase")]`.
 * This is the data contract between Rust and React.
 */
export interface FrameState {
  /** Current time in seconds. */
  time: number;
  /** Current frame number (0-indexed, u32 in Rust). */
  frame: number;
  /** Total duration of the timeline in seconds. */
  totalDuration: number;
  /** Frames per second (u32 in Rust). */
  fps: number;
  /** Visual state of all elements (both visible and hidden). */
  elements: ElementState[];
  /** Text of the currently active narration, or null if none. */
  activeNarration: string | null;
  /** Current theme as CSS custom properties. */
  theme: ThemeState;
  /**
   * Template name to use for rendering this frame.
   * Maps to a registered template component via the template registry.
   * Falls back to "default" (MoronFrame) if undefined or not found.
   *
   * Optional: Rust side does not send this field yet. When added to the
   * Rust FrameState struct, use `#[serde(skip_serializing_if = "Option::is_none")]`.
   */
  template?: string;
}

/**
 * Template registration system for Moron scenes.
 *
 * Templates are React components that receive FrameState and render a
 * complete frame layout. The registry maps template names to components,
 * allowing the host page to select the appropriate renderer at runtime.
 *
 * MoronFrame is pre-registered as the "default" template.
 */

import type { ComponentType } from "react";

import type { FrameState } from "../types";
import { MoronFrame } from "../MoronFrame";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * Props that every template component receives.
 *
 * Mirrors MoronFrameProps: a template is a drop-in replacement for
 * MoronFrame with a different visual layout.
 */
export interface TemplateProps {
  /** The complete visual state for this frame. */
  state: FrameState;
  /** Frame width in pixels. Defaults to 1920. */
  width?: number;
  /** Frame height in pixels. Defaults to 1080. */
  height?: number;
}

/**
 * A React component that can render a frame from FrameState.
 * All templates must conform to this signature.
 */
export type TemplateComponent = ComponentType<TemplateProps>;

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

const registry = new Map<string, TemplateComponent>();

/**
 * Register a template component under a given name.
 *
 * If a template with the same name is already registered, it is replaced
 * and a warning is logged. This is intentional: it allows hot-reloading
 * and plugin overrides during development.
 *
 * @param name - Unique template identifier (e.g. "explainer", "comparison")
 * @param component - React component conforming to TemplateComponent
 */
export function registerTemplate(
  name: string,
  component: TemplateComponent,
): void {
  if (registry.has(name)) {
    console.warn(
      `[moron] Template "${name}" is already registered. Replacing.`,
    );
  }
  registry.set(name, component);
}

/**
 * Retrieve a template component by name.
 *
 * Returns the registered component, or the "default" template (MoronFrame)
 * if the name is not found. This ensures rendering never fails due to a
 * missing template.
 *
 * @param name - Template name to look up
 * @returns The matching template component, or MoronFrame as fallback
 */
export function getTemplate(name: string): TemplateComponent {
  return registry.get(name) ?? registry.get("default")!;
}

/**
 * List all registered template names.
 *
 * Useful for tooling, gallery pages, and debugging.
 *
 * @returns Array of registered template name strings
 */
export function listTemplates(): string[] {
  return Array.from(registry.keys());
}

// ---------------------------------------------------------------------------
// Default registration
// ---------------------------------------------------------------------------

// MoronFrame is always available as the default/fallback template.
registerTemplate("default", MoronFrame);

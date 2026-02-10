/**
 * Theme definitions for the Moron motion graphics engine.
 *
 * Each theme is a record of CSS custom property names to values.
 * At render time the active theme's properties are injected into the
 * Chromium page via a <style> block before the React tree mounts.
 */

export interface MoronTheme {
  /** Human-readable name shown in tooling / gallery. */
  name: string;
  /** Path to the CSS file that defines the custom properties. */
  stylesheet: string;
}

/**
 * Built-in theme registry.
 * Custom themes can extend this map at runtime.
 */
export const themes: Record<string, MoronTheme> = {
  default: {
    name: "Default",
    stylesheet: new URL("./default.css", import.meta.url).href,
  },
  light: {
    name: "Light",
    stylesheet: new URL("./light.css", import.meta.url).href,
  },
};

export type ThemeName = keyof typeof themes;

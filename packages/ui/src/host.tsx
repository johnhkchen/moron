/**
 * Host page entry point for the Moron rendering pipeline.
 *
 * This file is the React application that ChromiumBridge loads via a file://
 * URL. It:
 *
 * 1. Mounts a React 19 root on the #root element
 * 2. Exposes window.__moron_setFrame(state) for the Chromium bridge
 * 3. On each call, looks up the template from the registry and re-renders
 *
 * The build script (build.mjs) bundles this file along with all dependencies
 * into a self-contained dist/index.html.
 */

import { createRoot } from "react-dom/client";
import { useState, useCallback, useEffect, useRef } from "react";

import type { FrameState } from "./types";
import { getTemplate } from "./templates";

// ---------------------------------------------------------------------------
// Global type augmentation
// ---------------------------------------------------------------------------

declare global {
  interface Window {
    __moron_setFrame: (state: FrameState) => void;
  }
}

// ---------------------------------------------------------------------------
// App component
// ---------------------------------------------------------------------------

/**
 * Root application component.
 *
 * Holds the current FrameState and renders the appropriate template.
 * The setFrame callback is exposed to the global scope so ChromiumBridge
 * can push new frames from Rust via CDP JavaScript evaluation.
 */
function App() {
  const [frameState, setFrameState] = useState<FrameState | null>(null);
  const setFrameRef = useRef<((state: FrameState) => void) | undefined>(undefined);

  // Stable callback that updates state. Stored in a ref so the global
  // function always points to the latest version without re-registration.
  const handleSetFrame = useCallback((state: FrameState) => {
    setFrameState(state);
  }, []);

  setFrameRef.current = handleSetFrame;

  // Expose the global function once on mount.
  useEffect(() => {
    window.__moron_setFrame = (state: FrameState) => {
      setFrameRef.current?.(state);
    };
  }, []);

  // Nothing to render until the first frame arrives.
  if (frameState === null) {
    return null;
  }

  // Look up the template. Falls back to "default" (MoronFrame) if the
  // template name is missing or not registered.
  const templateName = frameState.template ?? "default";
  const Template = getTemplate(templateName);

  return <Template state={frameState} />;
}

// ---------------------------------------------------------------------------
// Mount
// ---------------------------------------------------------------------------

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error(
    "[moron] Host page is missing a #root element. " +
      "The HTML template must contain <div id=\"root\"></div>.",
  );
}

const root = createRoot(rootElement);
root.render(<App />);

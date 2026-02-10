/**
 * Build script for @moron/ui host page.
 *
 * Bundles src/host.tsx and all its dependencies into a single self-contained
 * dist/index.html file that ChromiumBridge can load via a file:// URL.
 *
 * Usage: node build.mjs
 */

import * as esbuild from "esbuild";
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// ---------------------------------------------------------------------------
// Bundle the React application
// ---------------------------------------------------------------------------

const result = await esbuild.build({
  entryPoints: [resolve(__dirname, "src/host.tsx")],
  bundle: true,
  format: "esm",
  platform: "browser",
  target: "es2022",
  jsx: "automatic",
  write: false,
  minify: false,
  sourcemap: false,
  // Ensure all dependencies are bundled (no externals)
  packages: "bundle",
});

if (result.errors.length > 0) {
  console.error("Build failed:", result.errors);
  process.exit(1);
}

const bundledJs = result.outputFiles[0].text;

// ---------------------------------------------------------------------------
// Read the default theme CSS
// ---------------------------------------------------------------------------

const defaultCss = readFileSync(
  resolve(__dirname, "../themes/src/default.css"),
  "utf-8",
);

// ---------------------------------------------------------------------------
// Construct the self-contained HTML page
// ---------------------------------------------------------------------------

const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=1920, height=1080">
  <title>Moron Frame Renderer</title>
  <style>
    /* Reset */
    *, *::before, *::after {
      box-sizing: border-box;
    }
    html, body {
      margin: 0;
      padding: 0;
      width: 1920px;
      height: 1080px;
      overflow: hidden;
    }

    /* Default theme CSS custom properties */
${defaultCss}
  </style>
</head>
<body>
  <div id="root"></div>
  <script type="module">
${bundledJs}
  </script>
</body>
</html>
`;

// ---------------------------------------------------------------------------
// Write output
// ---------------------------------------------------------------------------

const distDir = resolve(__dirname, "dist");
mkdirSync(distDir, { recursive: true });

const outPath = resolve(distDir, "index.html");
writeFileSync(outPath, html, "utf-8");

const sizeKb = (Buffer.byteLength(html, "utf-8") / 1024).toFixed(1);
console.log(`Built ${outPath} (${sizeKb} KB)`);

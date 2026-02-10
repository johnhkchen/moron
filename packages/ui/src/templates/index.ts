/**
 * Template system for Moron scenes.
 *
 * Templates are higher-level compositions of base components that provide
 * ready-made layouts for common video patterns (title cards, listicles,
 * data dashboards, etc.).
 *
 * The registry maps template names to React components. MoronFrame is
 * pre-registered as the "default" template and serves as the fallback
 * when a requested template name is not found.
 */

export {
  registerTemplate,
  getTemplate,
  listTemplates,
} from "./registry";

export type { TemplateComponent, TemplateProps } from "./registry";

import type { Config } from "tailwindcss";

/**
 * Moron Tailwind theme contract.
 *
 * Bridges CSS custom properties defined in theme stylesheets (e.g. default.css)
 * to Tailwind utility classes. Swapping the CSS file swaps the entire visual
 * identity while keeping the same class names in templates.
 */
const config: Config = {
  content: ["../ui/src/**/*.{ts,tsx}"],

  theme: {
    extend: {
      colors: {
        moron: {
          bg: {
            primary: "var(--moron-bg-primary)",
            secondary: "var(--moron-bg-secondary)",
            tertiary: "var(--moron-bg-tertiary)",
          },
          fg: {
            primary: "var(--moron-fg-primary)",
            secondary: "var(--moron-fg-secondary)",
            muted: "var(--moron-fg-muted)",
          },
          accent: {
            DEFAULT: "var(--moron-accent)",
            hover: "var(--moron-accent-hover)",
            subtle: "var(--moron-accent-subtle)",
          },
          success: "var(--moron-success)",
          warning: "var(--moron-warning)",
          error: "var(--moron-error)",
        },
      },

      fontFamily: {
        sans: "var(--moron-font-sans)",
        mono: "var(--moron-font-mono)",
      },

      fontSize: {
        xs: "var(--moron-text-xs)",
        sm: "var(--moron-text-sm)",
        base: "var(--moron-text-base)",
        lg: "var(--moron-text-lg)",
        xl: "var(--moron-text-xl)",
        "2xl": "var(--moron-text-2xl)",
        "3xl": "var(--moron-text-3xl)",
        "4xl": "var(--moron-text-4xl)",
      },

      lineHeight: {
        tight: "var(--moron-leading-tight)",
        normal: "var(--moron-leading-normal)",
        relaxed: "var(--moron-leading-relaxed)",
      },

      fontWeight: {
        normal: "var(--moron-font-weight-normal)",
        medium: "var(--moron-font-weight-medium)",
        semibold: "var(--moron-font-weight-semibold)",
        bold: "var(--moron-font-weight-bold)",
      },

      spacing: {
        "moron-1": "var(--moron-space-1)",
        "moron-2": "var(--moron-space-2)",
        "moron-3": "var(--moron-space-3)",
        "moron-4": "var(--moron-space-4)",
        "moron-6": "var(--moron-space-6)",
        "moron-8": "var(--moron-space-8)",
        "moron-12": "var(--moron-space-12)",
        "moron-16": "var(--moron-space-16)",
        "moron-24": "var(--moron-space-24)",
      },

      borderRadius: {
        sm: "var(--moron-radius-sm)",
        md: "var(--moron-radius-md)",
        lg: "var(--moron-radius-lg)",
        full: "var(--moron-radius-full)",
      },

      transitionDuration: {
        instant: "var(--moron-duration-instant)",
        fast: "var(--moron-duration-fast)",
        normal: "var(--moron-duration-normal)",
        slow: "var(--moron-duration-slow)",
        slower: "var(--moron-duration-slower)",
      },

      transitionTimingFunction: {
        DEFAULT: "var(--moron-ease-default)",
        in: "var(--moron-ease-in)",
        out: "var(--moron-ease-out)",
        "in-out": "var(--moron-ease-in-out)",
        spring: "var(--moron-ease-spring)",
      },

      boxShadow: {
        sm: "var(--moron-shadow-sm)",
        md: "var(--moron-shadow-md)",
        lg: "var(--moron-shadow-lg)",
      },
    },
  },

  plugins: [],
};

export default config;

//! Default theme: built-in "moron" theme with sensible defaults.
//!
//! Every value here mirrors `packages/themes/src/default.css` so that the Rust
//! theme system and the CSS layer stay in sync.

use crate::theme::{
    Theme, ThemeColors, ThemeShadows, ThemeSpacing, ThemeTiming, ThemeTypography,
};

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            bg_primary: "#0f172a".into(),
            bg_secondary: "#1e293b".into(),
            bg_tertiary: "#334155".into(),

            fg_primary: "#f8fafc".into(),
            fg_secondary: "#cbd5e1".into(),
            fg_muted: "#64748b".into(),

            accent: "#3b82f6".into(),
            accent_hover: "#60a5fa".into(),
            accent_subtle: "rgba(59, 130, 246, 0.15)".into(),

            success: "#22c55e".into(),
            warning: "#eab308".into(),
            error: "#ef4444".into(),
        }
    }
}

impl Default for ThemeTypography {
    fn default() -> Self {
        Self {
            font_sans: r#""Inter", ui-sans-serif, system-ui, sans-serif"#.into(),
            font_mono: r#""JetBrains Mono", ui-monospace, monospace"#.into(),

            text_xs: "0.75rem".into(),
            text_sm: "0.875rem".into(),
            text_base: "1rem".into(),
            text_lg: "1.25rem".into(),
            text_xl: "1.5rem".into(),
            text_2xl: "2rem".into(),
            text_3xl: "2.5rem".into(),
            text_4xl: "3.5rem".into(),

            leading_tight: "1.15".into(),
            leading_normal: "1.5".into(),
            leading_relaxed: "1.75".into(),

            font_weight_normal: "400".into(),
            font_weight_medium: "500".into(),
            font_weight_semibold: "600".into(),
            font_weight_bold: "700".into(),
        }
    }
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            space_1: "0.25rem".into(),
            space_2: "0.5rem".into(),
            space_3: "0.75rem".into(),
            space_4: "1rem".into(),
            space_6: "1.5rem".into(),
            space_8: "2rem".into(),
            space_12: "3rem".into(),
            space_16: "4rem".into(),
            space_24: "6rem".into(),

            container_padding: "3rem".into(),

            radius_sm: "0.25rem".into(),
            radius_md: "0.5rem".into(),
            radius_lg: "1rem".into(),
            radius_full: "9999px".into(),
        }
    }
}

impl Default for ThemeTiming {
    fn default() -> Self {
        Self {
            duration_instant: "0ms".into(),
            duration_fast: "150ms".into(),
            duration_normal: "300ms".into(),
            duration_slow: "500ms".into(),
            duration_slower: "800ms".into(),

            ease_default: "cubic-bezier(0.4, 0, 0.2, 1)".into(),
            ease_in: "cubic-bezier(0.4, 0, 1, 1)".into(),
            ease_out: "cubic-bezier(0, 0, 0.2, 1)".into(),
            ease_in_out: "cubic-bezier(0.4, 0, 0.2, 1)".into(),
            ease_spring: "cubic-bezier(0.34, 1.56, 0.64, 1)".into(),
        }
    }
}

impl Default for ThemeShadows {
    fn default() -> Self {
        Self {
            shadow_sm: "0 1px 2px rgba(0, 0, 0, 0.3)".into(),
            shadow_md: "0 4px 6px rgba(0, 0, 0, 0.3)".into(),
            shadow_lg: "0 10px 15px rgba(0, 0, 0, 0.4)".into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "moron-dark".into(),
            colors: ThemeColors::default(),
            typography: ThemeTypography::default(),
            spacing: ThemeSpacing::default(),
            timing: ThemeTiming::default(),
            shadows: ThemeShadows::default(),
        }
    }
}

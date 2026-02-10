//! Theme trait and core theme data structures.
//!
//! The `Theme` struct is the bridge between Rust theme configuration and CSS
//! custom properties consumed by the React rendering layer.  Each sub-struct
//! maps 1-to-1 to a group of `--moron-*` CSS custom properties defined in
//! `packages/themes/src/default.css`.

use serde::{Deserialize, Serialize};

// ── Color tokens ────────────────────────────────────────────────────────────

/// All color tokens used by the theme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_tertiary: String,

    pub fg_primary: String,
    pub fg_secondary: String,
    pub fg_muted: String,

    pub accent: String,
    pub accent_hover: String,
    pub accent_subtle: String,

    pub success: String,
    pub warning: String,
    pub error: String,
}

// ── Typography tokens ───────────────────────────────────────────────────────

/// Font families, text sizes, line-heights, and font weights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeTypography {
    pub font_sans: String,
    pub font_mono: String,

    pub text_xs: String,
    pub text_sm: String,
    pub text_base: String,
    pub text_lg: String,
    pub text_xl: String,
    pub text_2xl: String,
    pub text_3xl: String,
    pub text_4xl: String,

    pub leading_tight: String,
    pub leading_normal: String,
    pub leading_relaxed: String,

    pub font_weight_normal: String,
    pub font_weight_medium: String,
    pub font_weight_semibold: String,
    pub font_weight_bold: String,
}

// ── Spacing tokens ──────────────────────────────────────────────────────────

/// Spacing scale and container padding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeSpacing {
    pub space_1: String,
    pub space_2: String,
    pub space_3: String,
    pub space_4: String,
    pub space_6: String,
    pub space_8: String,
    pub space_12: String,
    pub space_16: String,
    pub space_24: String,

    pub container_padding: String,

    pub radius_sm: String,
    pub radius_md: String,
    pub radius_lg: String,
    pub radius_full: String,
}

// ── Timing / animation tokens ───────────────────────────────────────────────

/// Durations and easing curves for animations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeTiming {
    pub duration_instant: String,
    pub duration_fast: String,
    pub duration_normal: String,
    pub duration_slow: String,
    pub duration_slower: String,

    pub ease_default: String,
    pub ease_in: String,
    pub ease_out: String,
    pub ease_in_out: String,
    pub ease_spring: String,
}

// ── Shadow tokens ───────────────────────────────────────────────────────────

/// Box-shadow tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeShadows {
    pub shadow_sm: String,
    pub shadow_md: String,
    pub shadow_lg: String,
}

// ── Theme (top-level) ───────────────────────────────────────────────────────

/// Complete theme definition.
///
/// A `Theme` contains every design token needed by the rendering layer.
/// Use [`Theme::to_css_properties`] to produce the CSS custom property pairs
/// that are injected into the page at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub timing: ThemeTiming,
    pub shadows: ThemeShadows,
}

impl Theme {
    /// Convert the theme into a list of CSS custom property key-value pairs.
    ///
    /// Each tuple is `("--moron-<token>", "<value>")`.
    pub fn to_css_properties(&self) -> Vec<(String, String)> {
        let mut props: Vec<(String, String)> = Vec::new();

        // Colors
        let c = &self.colors;
        props.push(("--moron-bg-primary".into(), c.bg_primary.clone()));
        props.push(("--moron-bg-secondary".into(), c.bg_secondary.clone()));
        props.push(("--moron-bg-tertiary".into(), c.bg_tertiary.clone()));
        props.push(("--moron-fg-primary".into(), c.fg_primary.clone()));
        props.push(("--moron-fg-secondary".into(), c.fg_secondary.clone()));
        props.push(("--moron-fg-muted".into(), c.fg_muted.clone()));
        props.push(("--moron-accent".into(), c.accent.clone()));
        props.push(("--moron-accent-hover".into(), c.accent_hover.clone()));
        props.push(("--moron-accent-subtle".into(), c.accent_subtle.clone()));
        props.push(("--moron-success".into(), c.success.clone()));
        props.push(("--moron-warning".into(), c.warning.clone()));
        props.push(("--moron-error".into(), c.error.clone()));

        // Typography
        let t = &self.typography;
        props.push(("--moron-font-sans".into(), t.font_sans.clone()));
        props.push(("--moron-font-mono".into(), t.font_mono.clone()));
        props.push(("--moron-text-xs".into(), t.text_xs.clone()));
        props.push(("--moron-text-sm".into(), t.text_sm.clone()));
        props.push(("--moron-text-base".into(), t.text_base.clone()));
        props.push(("--moron-text-lg".into(), t.text_lg.clone()));
        props.push(("--moron-text-xl".into(), t.text_xl.clone()));
        props.push(("--moron-text-2xl".into(), t.text_2xl.clone()));
        props.push(("--moron-text-3xl".into(), t.text_3xl.clone()));
        props.push(("--moron-text-4xl".into(), t.text_4xl.clone()));
        props.push(("--moron-leading-tight".into(), t.leading_tight.clone()));
        props.push(("--moron-leading-normal".into(), t.leading_normal.clone()));
        props.push(("--moron-leading-relaxed".into(), t.leading_relaxed.clone()));
        props.push(("--moron-font-weight-normal".into(), t.font_weight_normal.clone()));
        props.push(("--moron-font-weight-medium".into(), t.font_weight_medium.clone()));
        props.push(("--moron-font-weight-semibold".into(), t.font_weight_semibold.clone()));
        props.push(("--moron-font-weight-bold".into(), t.font_weight_bold.clone()));

        // Spacing
        let s = &self.spacing;
        props.push(("--moron-space-1".into(), s.space_1.clone()));
        props.push(("--moron-space-2".into(), s.space_2.clone()));
        props.push(("--moron-space-3".into(), s.space_3.clone()));
        props.push(("--moron-space-4".into(), s.space_4.clone()));
        props.push(("--moron-space-6".into(), s.space_6.clone()));
        props.push(("--moron-space-8".into(), s.space_8.clone()));
        props.push(("--moron-space-12".into(), s.space_12.clone()));
        props.push(("--moron-space-16".into(), s.space_16.clone()));
        props.push(("--moron-space-24".into(), s.space_24.clone()));
        props.push(("--moron-container-padding".into(), s.container_padding.clone()));
        props.push(("--moron-radius-sm".into(), s.radius_sm.clone()));
        props.push(("--moron-radius-md".into(), s.radius_md.clone()));
        props.push(("--moron-radius-lg".into(), s.radius_lg.clone()));
        props.push(("--moron-radius-full".into(), s.radius_full.clone()));

        // Timing
        let tm = &self.timing;
        props.push(("--moron-duration-instant".into(), tm.duration_instant.clone()));
        props.push(("--moron-duration-fast".into(), tm.duration_fast.clone()));
        props.push(("--moron-duration-normal".into(), tm.duration_normal.clone()));
        props.push(("--moron-duration-slow".into(), tm.duration_slow.clone()));
        props.push(("--moron-duration-slower".into(), tm.duration_slower.clone()));
        props.push(("--moron-ease-default".into(), tm.ease_default.clone()));
        props.push(("--moron-ease-in".into(), tm.ease_in.clone()));
        props.push(("--moron-ease-out".into(), tm.ease_out.clone()));
        props.push(("--moron-ease-in-out".into(), tm.ease_in_out.clone()));
        props.push(("--moron-ease-spring".into(), tm.ease_spring.clone()));

        // Shadows
        let sh = &self.shadows;
        props.push(("--moron-shadow-sm".into(), sh.shadow_sm.clone()));
        props.push(("--moron-shadow-md".into(), sh.shadow_md.clone()));
        props.push(("--moron-shadow-lg".into(), sh.shadow_lg.clone()));

        props
    }
}

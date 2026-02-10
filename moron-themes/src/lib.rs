//! moron-themes: Theme system and CSS custom properties contract.
//!
//! Defines the bridge between Rust theme configuration and Tailwind CSS theming.

pub mod defaults;
pub mod theme;

// Re-export all public types at crate root for convenience.
pub use theme::{Theme, ThemeColors, ThemeShadows, ThemeSpacing, ThemeTiming, ThemeTypography};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_non_empty_values() {
        let theme = Theme::default();
        assert!(!theme.name.is_empty());
        assert!(!theme.colors.bg_primary.is_empty());
        assert!(!theme.colors.accent.is_empty());
        assert!(!theme.typography.font_sans.is_empty());
        assert!(!theme.typography.text_base.is_empty());
        assert!(!theme.spacing.space_4.is_empty());
        assert!(!theme.timing.duration_normal.is_empty());
        assert!(!theme.shadows.shadow_md.is_empty());
    }

    #[test]
    fn to_css_properties_returns_expected_pairs() {
        let theme = Theme::default();
        let props = theme.to_css_properties();

        // Collect into a HashMap for easy lookup.
        let map: std::collections::HashMap<String, String> = props.into_iter().collect();

        // Test specific color property
        assert_eq!(map.get("--moron-bg-primary").unwrap(), "#0f172a");

        // Test specific typography property
        assert_eq!(map.get("--moron-text-base").unwrap(), "1rem");

        // Test specific accent property
        assert_eq!(map.get("--moron-accent").unwrap(), "#3b82f6");

        // Test specific spacing property
        assert_eq!(map.get("--moron-space-8").unwrap(), "2rem");

        // Test specific timing property
        assert_eq!(map.get("--moron-duration-fast").unwrap(), "150ms");

        // Test specific shadow property
        assert_eq!(
            map.get("--moron-shadow-sm").unwrap(),
            "0 1px 2px rgba(0, 0, 0, 0.3)"
        );
    }

    #[test]
    fn to_css_properties_covers_all_tokens() {
        let theme = Theme::default();
        let props = theme.to_css_properties();

        // Every property key must start with "--moron-"
        for (key, value) in &props {
            assert!(
                key.starts_with("--moron-"),
                "CSS property key '{key}' does not start with --moron-"
            );
            assert!(!value.is_empty(), "CSS property '{key}' has empty value");
        }

        // We expect at least 50 properties (12 colors + 17 typography + 14 spacing + 10 timing + 3 shadows = 56)
        assert!(
            props.len() >= 50,
            "Expected at least 50 CSS properties, got {}",
            props.len()
        );
    }

    #[test]
    fn serde_round_trip() {
        let theme = Theme::default();
        let json = serde_json::to_string_pretty(&theme).expect("serialize to JSON");
        let deserialized: Theme = serde_json::from_str(&json).expect("deserialize from JSON");
        assert_eq!(theme, deserialized);
    }

    #[test]
    fn serde_json_contains_expected_fields() {
        let theme = Theme::default();
        let value: serde_json::Value =
            serde_json::to_value(&theme).expect("serialize to serde_json::Value");

        assert_eq!(value["name"], "moron-dark");
        assert_eq!(value["colors"]["bg_primary"], "#0f172a");
        assert_eq!(value["typography"]["font_weight_bold"], "700");
        assert_eq!(value["spacing"]["radius_full"], "9999px");
        assert_eq!(value["timing"]["ease_spring"], "cubic-bezier(0.34, 1.56, 0.64, 1)");
        assert_eq!(value["shadows"]["shadow_lg"], "0 10px 15px rgba(0, 0, 0, 0.4)");
    }

    // -- Light theme tests -------------------------------------------------

    #[test]
    fn light_theme_has_correct_name() {
        let theme = Theme::light();
        assert_eq!(theme.name, "moron-light");
    }

    #[test]
    fn light_theme_has_non_empty_values() {
        let theme = Theme::light();
        assert!(!theme.name.is_empty());
        assert!(!theme.colors.bg_primary.is_empty());
        assert!(!theme.colors.accent.is_empty());
        assert!(!theme.typography.font_sans.is_empty());
        assert!(!theme.typography.text_base.is_empty());
        assert!(!theme.spacing.space_4.is_empty());
        assert!(!theme.timing.duration_normal.is_empty());
        assert!(!theme.shadows.shadow_md.is_empty());
    }

    #[test]
    fn light_theme_differs_from_default() {
        let dark = Theme::default();
        let light = Theme::light();

        assert_ne!(dark.name, light.name);
        assert_ne!(dark.colors.bg_primary, light.colors.bg_primary);
        assert_ne!(dark.colors.fg_primary, light.colors.fg_primary);
        assert_ne!(dark.colors.accent, light.colors.accent);
        assert_ne!(dark.shadows.shadow_sm, light.shadows.shadow_sm);
    }

    #[test]
    fn light_theme_to_css_properties_count() {
        let theme = Theme::light();
        let props = theme.to_css_properties();

        assert_eq!(
            props.len(),
            56,
            "Light theme must produce exactly 56 CSS properties, got {}",
            props.len()
        );

        for (key, value) in &props {
            assert!(
                key.starts_with("--moron-"),
                "CSS property key '{key}' does not start with --moron-"
            );
            assert!(!value.is_empty(), "CSS property '{key}' has empty value");
        }
    }

    #[test]
    fn light_theme_serde_round_trip() {
        let theme = Theme::light();
        let json = serde_json::to_string_pretty(&theme).expect("serialize to JSON");
        let deserialized: Theme = serde_json::from_str(&json).expect("deserialize from JSON");
        assert_eq!(theme, deserialized);
    }
}

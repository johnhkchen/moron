//! Integration tests verifying that CSS theme files and Rust theme constructors
//! produce identical sets of CSS custom property key-value pairs.
//!
//! These tests read the CSS files from `packages/themes/src/` at test time and
//! compare against the output of `Theme::to_css_properties()`. If a value is
//! added, removed, or changed on one side but not the other, these tests fail.

use std::collections::HashMap;

use moron_themes::Theme;

// ---------------------------------------------------------------------------
// CSS parser (minimal, purpose-built)
// ---------------------------------------------------------------------------

/// Parse `--moron-*: value;` declarations from a CSS file's `:root` block.
///
/// Handles the well-structured CSS files in `packages/themes/src/`. Not a
/// general-purpose CSS parser — only extracts custom property declarations.
fn parse_css_custom_properties(css: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();

    for line in css.lines() {
        let trimmed = line.trim();

        // Skip comments, empty lines, selectors, closing braces.
        if trimmed.is_empty()
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with(':')
            || trimmed.starts_with('}')
        {
            continue;
        }

        // Match lines like `--moron-bg-primary: #0f172a;`
        if let Some(colon_pos) = trimmed.find(": ") {
            let key = trimmed[..colon_pos].trim();
            if key.starts_with("--moron-") {
                let value = trimmed[colon_pos + 2..].trim();
                // Strip trailing semicolon.
                let value = value.strip_suffix(';').unwrap_or(value).trim();
                props.insert(key.to_string(), value.to_string());
            }
        }
    }

    props
}

/// Build the path to a theme CSS file relative to the crate manifest directory.
fn css_file_path(filename: &str) -> String {
    format!(
        "{}/../packages/themes/src/{}",
        env!("CARGO_MANIFEST_DIR"),
        filename
    )
}

/// Read and parse a theme CSS file, returning its custom properties.
fn load_css_properties(filename: &str) -> HashMap<String, String> {
    let path = css_file_path(filename);
    let css = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    parse_css_custom_properties(&css)
}

/// Build a HashMap from a Theme's to_css_properties() output.
fn rust_properties(theme: &Theme) -> HashMap<String, String> {
    theme.to_css_properties().into_iter().collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// The `--moron-container-padding` property uses a `var()` reference in CSS
/// (`var(--moron-space-12)`) but Rust stores the resolved value (`3rem`).
/// This is intentional: CSS cascades the reference while Rust needs a concrete
/// value. We skip this property in value comparisons.
const SKIP_VALUE_COMPARE: &[&str] = &["--moron-container-padding"];

#[test]
fn default_css_matches_rust_defaults() {
    let css_props = load_css_properties("default.css");
    let rust_props = rust_properties(&Theme::default());

    // Same set of keys.
    let mut css_keys: Vec<&String> = css_props.keys().collect();
    let mut rust_keys: Vec<&String> = rust_props.keys().collect();
    css_keys.sort();
    rust_keys.sort();
    assert_eq!(
        css_keys, rust_keys,
        "default.css and Theme::default() have different property keys"
    );

    // Same values (except skipped properties).
    for (key, css_val) in &css_props {
        if SKIP_VALUE_COMPARE.contains(&key.as_str()) {
            continue;
        }
        let rust_val = rust_props.get(key).unwrap();
        assert_eq!(
            css_val, rust_val,
            "Property {key} differs: CSS={css_val:?}, Rust={rust_val:?}"
        );
    }
}

#[test]
fn light_css_matches_rust_light() {
    let css_props = load_css_properties("light.css");
    let rust_props = rust_properties(&Theme::light());

    // Same set of keys.
    let mut css_keys: Vec<&String> = css_props.keys().collect();
    let mut rust_keys: Vec<&String> = rust_props.keys().collect();
    css_keys.sort();
    rust_keys.sort();
    assert_eq!(
        css_keys, rust_keys,
        "light.css and Theme::light() have different property keys"
    );

    // Same values (except skipped properties).
    for (key, css_val) in &css_props {
        if SKIP_VALUE_COMPARE.contains(&key.as_str()) {
            continue;
        }
        let rust_val = rust_props.get(key).unwrap();
        assert_eq!(
            css_val, rust_val,
            "Property {key} differs: CSS={css_val:?}, Rust={rust_val:?}"
        );
    }
}

#[test]
fn css_files_have_56_properties() {
    let default_props = load_css_properties("default.css");
    let light_props = load_css_properties("light.css");

    assert_eq!(
        default_props.len(),
        56,
        "default.css should have exactly 56 properties, got {}",
        default_props.len()
    );
    assert_eq!(
        light_props.len(),
        56,
        "light.css should have exactly 56 properties, got {}",
        light_props.len()
    );
}

#[test]
fn both_css_files_define_same_property_keys() {
    let default_props = load_css_properties("default.css");
    let light_props = load_css_properties("light.css");

    let mut default_keys: Vec<&String> = default_props.keys().collect();
    let mut light_keys: Vec<&String> = light_props.keys().collect();
    default_keys.sort();
    light_keys.sort();

    assert_eq!(
        default_keys, light_keys,
        "default.css and light.css must define the same set of property keys"
    );
}

#[test]
fn parser_handles_css_comments_and_blank_lines() {
    let css = r#"
/* A comment */
:root {
  /* Section header */
  --moron-test-a: value-a;
  --moron-test-b: value-b;

  /* Another comment */
  --moron-test-c: value c with spaces;
}
"#;
    let props = parse_css_custom_properties(css);
    assert_eq!(props.len(), 3);
    assert_eq!(props.get("--moron-test-a").unwrap(), "value-a");
    assert_eq!(props.get("--moron-test-b").unwrap(), "value-b");
    assert_eq!(props.get("--moron-test-c").unwrap(), "value c with spaces");
}

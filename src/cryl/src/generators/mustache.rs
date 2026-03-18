use std::collections::HashMap;
use std::path::Path;

use crate::common::{
  deserialize, read_file_if_exists, save_atomic, CrylResult, Format,
};

/// Mustache template input structure
#[derive(serde::Deserialize)]
struct MustacheInput {
  template: String,
  variables: HashMap<String, String>,
}

/// Generate a populated Mustache template
///
/// # Arguments
/// * `name` - Path to save the generated file
/// * `format` - Input format (json, yaml, toml)
/// * `variables_and_template` - Path to file containing template and variables
/// * `renew` - Overwrite destination if it exists
///
/// # Description
/// Reads a file containing a template string and a map of variables. Each
/// variable value is either used directly or, if it exists as a file path,
/// the file content is read and used as the value. The template is then
/// rendered using Mustache templating.
pub fn generate_mustache(
  name: &Path,
  format: &str,
  variables_and_template: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Read and deserialize the input file
  let format = Format::parse(format)?;
  let input_content = std::fs::read_to_string(variables_and_template)?;
  let input: MustacheInput = deserialize(&input_content, format)?;

  // Process each variable
  let mut context: HashMap<String, String> = HashMap::new();
  for (key, value) in input.variables {
    // Check if value is a file path and read it if so
    let raw_value = if let Some(content) = read_file_if_exists(&value)? {
      content
    } else {
      value
    };

    // Trim whitespace (matching nushell implementation)
    let trimmed = raw_value.trim();

    context.insert(key, trimmed.to_string());
  }

  // Parse and render the template
  let template = mustache::compile_str(&input.template)?;
  let rendered = template.render_to_string(&context)?;

  // Save the file (private permissions since it may contain secrets)
  save_atomic(name, rendered.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_mustache_basic() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    let input = serde_json::json!({
      "template": "Hello {{name}}!",
      "variables": {
        "name": "World"
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  fn test_generate_mustache_reads_file_values() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let secret_path = temp.path().join("secret.txt");
    let output_path = temp.path().join("output.txt");

    fs::write(&secret_path, "my secret").unwrap();

    let input = serde_json::json!({
      "template": "Password: {{password}}",
      "variables": {
        "password": secret_path.to_str().unwrap()
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Password: my secret");
  }

  #[test]
  fn test_generate_mustache_trims_whitespace() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    let input = serde_json::json!({
      "template": "Value: '{{value}}'",
      "variables": {
        "value": "  trimmed  "
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Value: 'trimmed'");
  }

  #[test]
  fn test_generate_mustache_multiple_variables() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    let input = serde_json::json!({
      "template": "{{greeting}}, {{name}}! You have {{count}} messages.",
      "variables": {
        "greeting": "Hello",
        "name": "User",
        "count": "5"
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello, User! You have 5 messages.");
  }

  #[test]
  fn test_generate_mustache_yaml_format() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.yaml");
    let output_path = temp.path().join("output.txt");

    let yaml_content = r#"
template: "Hello {{name}}!"
variables:
  name: World
"#;
    fs::write(&input_path, yaml_content).unwrap();

    generate_mustache(&output_path, "yaml", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  fn test_generate_mustache_toml_format() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.toml");
    let output_path = temp.path().join("output.txt");

    let toml_content = r#"
template = "Hello {{name}}!"

[variables]
name = "World"
"#;
    fs::write(&input_path, toml_content).unwrap();

    generate_mustache(&output_path, "toml", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  fn test_generate_mustache_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    fs::write(&output_path, "original content").unwrap();

    let input = serde_json::json!({
      "template": "New {{value}}",
      "variables": {
        "value": "content"
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "original content");
  }

  #[test]
  fn test_generate_mustache_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    fs::write(&output_path, "original content").unwrap();

    let input = serde_json::json!({
      "template": "New {{value}}",
      "variables": {
        "value": "content"
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, true).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "New content");
  }

  #[test]
  fn test_generate_mustache_private_permissions() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    let input = serde_json::json!({
      "template": "test",
      "variables": {}
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let metadata = fs::metadata(&output_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_generate_mustache_missing_variable() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    let input = serde_json::json!({
      "template": "Hello {{missing}}!",
      "variables": {}
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    // Missing variables render as empty string in mustache
    assert_eq!(content, "Hello !");
  }

  #[test]
  fn test_generate_mustache_multiline_file_content() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let content_path = temp.path().join("content.txt");
    let output_path = temp.path().join("output.txt");

    fs::write(&content_path, "line1\nline2\nline3").unwrap();

    let input = serde_json::json!({
      "template": "Content:\n{{data}}",
      "variables": {
        "data": content_path.to_str().unwrap()
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Content:\nline1\nline2\nline3");
  }

  #[test]
  fn test_generate_mustache_sections() {
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("output.txt");

    let input = serde_json::json!({
      "template": "{{#show}}Visible{{/show}}{{#hide}}Hidden{{/hide}}",
      "variables": {
        "show": "true",
        "hide": ""
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, "json", &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Visible");
  }
}

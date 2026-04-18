use itertools::Itertools;
use std::collections::HashMap;
use std::path::Path;

use crate::common::{
  CrylResult, DirectoryListing, Format, deserialize_from_file, list_directory,
  save_atomic,
};

/// Mustache template input structure
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MustacheInput {
  pub template: String,
  pub listing: DirectoryListing,
}

/// Generate a populated Mustache template
///
/// # Arguments
/// * `name` - Path to save the generated file
/// * `format` - Format of listing and template
/// * `listing_and_template` - Path to file containing template and listing
/// * `renew` - Overwrite destination if it exists
///
/// # Description
/// Reads a file containing a template string and a directory listing.
/// Files are mapped
///  Each
/// variable value is either used directly or, if it exists as a file path,
/// the file content is read and used as the value. The template is then
/// rendered using Mustache templating.
pub fn generate_mustache(
  name: &Path,
  format: Format,
  listing_and_template: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Read and deserialize the input file
  let input: MustacheInput =
    deserialize_from_file(listing_and_template, Some(format))?;

  // Keep in variable here so we can consume it with list_directory
  let is_map = matches!(input.listing, DirectoryListing::Map(_));

  // Process each variable
  let mut context: HashMap<String, String> = HashMap::new();
  for (key, value) in list_directory(std::env::current_dir()?, input.listing)? {
    // Convert paths to screaming snake case
    let key = if !is_map {
      Path::new(&key)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_uppercase())
        .join("_")
    } else {
      key
    };

    context.insert(key, std::fs::read_to_string(value)?);
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
  use serial_test::serial;

  use super::*;
  use crate::common::TempCurrentDir;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use std::path::PathBuf;
  use std::str::FromStr;

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_basic() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write("world", "World").unwrap();
    let input = serde_json::json!({
      "template": "Hello {{name}}!",
      "listing": {
        "type": "map",
        "value": {
          "name": "world"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_missing_file_fails() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let secret_path = PathBuf::from_str("secret.txt").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    let input = serde_json::json!({
      "template": "Password: {{password}}",
      "listing": {
        "type": "map",
        "values": {
          "password": secret_path.to_str().unwrap()
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    let result =
      generate_mustache(&output_path, Format::Json, &input_path, false);
    assert!(matches!(result, Err(_)));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_does_not_trim_whitespace() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write("value", "  not trimmed  ").unwrap();
    let input = serde_json::json!({
      "template": "Value: '{{value}}'",
      "listing": {
        "type": "map",
        "value": {
          "value": "value"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Value: '  not trimmed  '");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_multiple_variables() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write("greeting", "Hello").unwrap();
    fs::write("name", "User").unwrap();
    fs::write("count", "5").unwrap();
    let input = serde_json::json!({
      "template": "{{greeting}}, {{name}}! You have {{count}} messages.",
      "listing": {
        "type": "map",
        "value": {
          "greeting": "greeting",
          "name": "name",
          "count": "count"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello, User! You have 5 messages.");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_yaml_format() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.yaml").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write("name", "World").unwrap();
    let yaml_content = r#"
template: "Hello {{name}}!"
listing:
  type: map
  value:
    name: name
"#;
    fs::write(&input_path, yaml_content).unwrap();

    generate_mustache(&output_path, Format::Yaml, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_toml_format() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.toml").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write("name", "World").unwrap();
    let toml_content = r#"
template = "Hello {{name}}!"

[listing]
type = "map"

[listing.value]
name = "name"
"#;
    fs::write(&input_path, toml_content).unwrap();

    generate_mustache(&output_path, Format::Toml, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_renew_false_no_overwrite() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write(&output_path, "original content").unwrap();

    fs::write("value", "new content").unwrap();
    let input = serde_json::json!({
      "template": "New {{value}}",
      "listing": {
        "type": "map",
        "value": {
          "value": "value"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "original content");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_renew_true_overwrites() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write(&output_path, "original content").unwrap();

    fs::write("value", "content").unwrap();
    let input = serde_json::json!({
      "template": "New {{value}}",
      "listing": {
        "type": "map",
        "value": {
          "value": "value"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, true).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "New content");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_private_permissions() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    let input = serde_json::json!({
      "template": "test",
      "listing": {
        "type": "map",
        "value": {}
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let metadata = fs::metadata(&output_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_missing_variable() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    let input = serde_json::json!({
      "template": "Hello {{missing}}!",
      "listing": {
        "type": "map",
        "value": {}
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    // Missing variables render as empty string in mustache
    assert_eq!(content, "Hello !");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_multiline_file_content() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let content_path = PathBuf::from_str("content.txt").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write(&content_path, "line1\nline2\nline3").unwrap();
    let input = serde_json::json!({
      "template": "Content:\n{{data}}",
      "listing": {
        "type": "map",
        "value": {
          "data": content_path.to_str().unwrap()
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Content:\nline1\nline2\nline3");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_sections() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::write("show", "true").unwrap();
    fs::write("hide", "").unwrap();
    let input = serde_json::json!({
      "template": "{{#show}}Visible{{/show}}{{#hide}}Hidden{{/hide}}",
      "listing": {
        "type": "map",
        "value": {
          "show": "show",
          "hide": "hide"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Visible");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_input_subdir() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("subdir").unwrap().join("input.json");
    let output_path = PathBuf::from_str("output.txt").unwrap();

    fs::create_dir_all("subdir2").unwrap();
    fs::write("subdir2/name", "World").unwrap();
    let input = serde_json::json!({
      "template": "Hello {{name}}!",
      "listing": {
        "type": "map",
        "value": {
          "name": "subdir2/name"
        }
      }
    });
    fs::create_dir_all(input_path.parent().unwrap()).unwrap();
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_mustache_output_subdir() {
    let _temp = TempCurrentDir::new().unwrap();
    let input_path = PathBuf::from_str("input.json").unwrap();
    let output_path = PathBuf::from_str("subdir").unwrap().join("output.txt");

    fs::write("name", "World").unwrap();
    let input = serde_json::json!({
      "template": "Hello {{name}}!",
      "listing": {
        "type": "map",
        "value": {
          "name": "name"
        }
      }
    });
    fs::write(&input_path, input.to_string()).unwrap();

    generate_mustache(&output_path, Format::Json, &input_path, false).unwrap();

    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "Hello World!");
  }
}

use crate::cli::*;
use crate::common::{CrylError, CrylResult, Format, deserialize};
use crate::dispatch::*;
use crate::manifest::Manifest;
use crate::schema::*;
use crate::versions::tool_versions;
use crate::{exporters, generators, importers};
use clap::Parser;
use schemars::schema_for;
use std::io::{self, Read};
use std::path::Path;

pub fn print_schema() -> CrylResult<()> {
  let schema = schema_for!(Specification);
  println!("{}", serde_json::to_string_pretty(&schema)?);
  Ok(())
}

pub fn run_from_path(
  spec_path: &Path,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
) -> CrylResult<()> {
  let format = Format::detect_from_path(spec_path)?;
  let content = std::fs::read_to_string(spec_path)?;

  // Validate specification size
  if content.len() > common.max_specification_size {
    return Err(CrylError::Validation(format!(
      "Specification size ({} bytes) exceeds maximum allowed ({} bytes)",
      content.len(),
      common.max_specification_size
    )));
  }

  let spec: Specification = deserialize(&content, format)?;

  run(&spec, common, sandbox, Some(spec_path), &content, format)?;

  Ok(())
}

pub fn run_from_stdin(
  format: &str,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
) -> CrylResult<()> {
  let format = Format::parse(format)?;
  let mut content = String::new();
  io::stdin().read_to_string(&mut content)?;

  // Validate specification size
  if content.len() > common.max_specification_size {
    return Err(CrylError::Validation(format!(
      "Specification size ({} bytes) exceeds maximum allowed ({} bytes)",
      content.len(),
      common.max_specification_size
    )));
  }

  let spec: Specification = deserialize(&content, format)?;

  run(&spec, common, sandbox, None, &content, format)?;

  Ok(())
}

fn run(
  spec: &Specification,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
  spec_path: Option<&Path>,
  spec_content: &str,
  spec_format: Format,
) -> CrylResult<()> {
  // Check if we need to enter sandbox mode
  if !sandbox.nosandbox && std::env::var("CRYL_SANDBOX").is_err() {
    return run_sandbox(
      spec,
      common,
      sandbox,
      spec_path,
      spec_content,
      spec_format,
    );
  }

  // Create manifest if not disabled
  let mut manifest = if common.no_manifest {
    None
  } else {
    Some(Manifest::new(spec_content, spec_format))
  };

  // Validate import count
  if spec.imports.len() > common.max_imports {
    return Err(CrylError::Validation(format!(
      "Import count ({}) exceeds maximum allowed ({})",
      spec.imports.len(),
      common.max_imports
    )));
  }

  // Validate generation count
  if spec.generations.len() > common.max_generations {
    return Err(CrylError::Validation(format!(
      "Generation count ({}) exceeds maximum allowed ({})",
      spec.generations.len(),
      common.max_generations
    )));
  }

  // Validate export count
  if spec.exports.len() > common.max_exports {
    return Err(CrylError::Validation(format!(
      "Export count ({}) exceeds maximum allowed ({})",
      spec.exports.len(),
      common.max_exports
    )));
  }

  for import in spec.imports.iter() {
    run_import_spec(import)?;
  }

  for generation in spec.generations.iter() {
    run_generate_spec(generation, common.allow_script)?;
  }

  if common.dry_run {
    return Ok(());
  }

  for export in spec.exports.iter() {
    run_export_spec(export)?;
  }

  // Save manifest on successful completion
  if let Some(mut manifest) = manifest {
    // Record all tools that are available
    for tool in tool_versions().keys() {
      manifest.record_tool(tool);
    }

    // Record all output files
    manifest.record_all_outputs()?;

    // Parse manifest format from string
    let manifest_format = Format::parse(&common.manifest_format)?;
    manifest.save(manifest_format)?;
  }

  Ok(())
}

fn run_sandbox(
  _spec: &Specification,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
  spec_path: Option<&Path>,
  spec_content: &str,
  spec_format: Format,
) -> CrylResult<()> {
  use std::process::Command;

  // Get path to current cryl binary
  let current_exe = std::env::current_exe().map_err(|e| {
    CrylError::Sandbox(format!("Failed to get current executable: {}", e))
  })?;

  // Create a temp directory for the spec file if needed
  let temp_dir = tempfile::tempdir().map_err(|e| {
    CrylError::Sandbox(format!("Failed to create temp directory: {}", e))
  })?;

  // Determine spec file path inside sandbox
  let sandbox_spec_path = if let Some(path) = spec_path {
    // If spec is from a file, we need to copy it to temp dir and mount it
    let file_name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("spec.toml");
    let temp_spec = temp_dir.path().join(file_name);
    std::fs::write(&temp_spec, spec_content).map_err(|e| {
      CrylError::Sandbox(format!("Failed to write spec to temp: {}", e))
    })?;
    temp_spec
  } else {
    // If spec is from stdin, write to temp file
    let temp_spec = temp_dir.path().join("spec");
    std::fs::write(&temp_spec, spec_content).map_err(|e| {
      CrylError::Sandbox(format!("Failed to write spec to temp: {}", e))
    })?;
    temp_spec
  };

  // Build bwrap arguments
  let mut bwrap_args: Vec<String> = vec![
    "--clearenv".to_string(),
    "--setenv".to_string(),
    "CRYL_SANDBOX".to_string(),
    "1".to_string(),
    "--setenv".to_string(),
    "LC_ALL".to_string(),
    "C.UTF-8".to_string(),
    "--setenv".to_string(),
    "LANG".to_string(),
    "C.UTF-8".to_string(),
    "--tmpfs".to_string(),
    "/work".to_string(),
    "--chdir".to_string(),
    "/work".to_string(),
    "--proc".to_string(),
    "/proc".to_string(),
    "--dev".to_string(),
    "/dev".to_string(),
    "--tmpfs".to_string(),
    "/tmp".to_string(),
    "--setenv".to_string(),
    "TMPDIR".to_string(),
    "/tmp".to_string(),
    "--dir".to_string(),
    "/home".to_string(),
    "--setenv".to_string(),
    "HOME".to_string(),
    "/home".to_string(),
    "--unshare-user".to_string(),
    "--uid".to_string(),
    "0".to_string(),
    "--gid".to_string(),
    "0".to_string(),
    "--unshare-pid".to_string(),
    "--unshare-uts".to_string(),
    "--unshare-ipc".to_string(),
  ];

  // Network isolation (unless allowed)
  if !sandbox.allow_net {
    bwrap_args.push("--unshare-net".to_string());
  }

  // Mount /nix/store as read-only (required for NixOS and binaries to work)
  if Path::new("/nix/store").exists() {
    bwrap_args.push("--ro-bind".to_string());
    bwrap_args.push("/nix/store".to_string());
    bwrap_args.push("/nix/store".to_string());
  }

  // Create /bin directory and symlink cryl there for convenience
  bwrap_args.push("--dir".to_string());
  bwrap_args.push("/bin".to_string());

  // Also bind mount the current exe in case it's not in /nix/store (e.g., during development)
  let current_exe_str = current_exe.to_string_lossy().to_string();
  if !current_exe_str.starts_with("/nix/store/") {
    bwrap_args.push("--ro-bind".to_string());
    bwrap_args.push(current_exe_str.clone());
    bwrap_args.push("/bin/cryl".to_string());
  }

  // Mount the spec file as read-only with proper extension
  let spec_file_name = sandbox_spec_path
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("spec.toml");
  let sandbox_spec_target = format!("/spec/{}", spec_file_name);
  bwrap_args.push("--ro-bind".to_string());
  bwrap_args.push(sandbox_spec_path.to_string_lossy().to_string());
  bwrap_args.push(sandbox_spec_target.clone());

  // Handle ro_binds
  for bind in &sandbox.ro_binds {
    let abs_path = std::fs::canonicalize(bind).map_err(|e| {
      CrylError::Sandbox(format!(
        "Failed to canonicalize ro_bind '{}': {}",
        bind.display(),
        e
      ))
    })?;
    bwrap_args.push("--ro-bind".to_string());
    bwrap_args.push(abs_path.to_string_lossy().to_string());
    bwrap_args.push(abs_path.to_string_lossy().to_string());
  }

  // Handle binds (read-write)
  for bind in &sandbox.binds {
    let abs_path = std::fs::canonicalize(bind).map_err(|e| {
      CrylError::Sandbox(format!(
        "Failed to canonicalize bind '{}': {}",
        bind.display(),
        e
      ))
    })?;
    bwrap_args.push("--bind".to_string());
    bwrap_args.push(abs_path.to_string_lossy().to_string());
    bwrap_args.push(abs_path.to_string_lossy().to_string());
  }

  // Create a temp dir for tools and link required tools
  let tools_dir = temp_dir.path().join("tools");
  std::fs::create_dir(&tools_dir).map_err(|e| {
    CrylError::Sandbox(format!("Failed to create tools dir: {}", e))
  })?;

  // Always include basic tools
  let basic_tools = vec![
    "openssl",
    "age-keygen",
    "sops",
    "ssh-keygen",
    "wg",
    "vault",
    "medusa",
    "nu",
    "argon2",
    "mkpasswd",
    "cockroach",
    "nebula-cert",
    "ssss-split",
    "ssss-combine",
  ];

  for tool in &basic_tools {
    if let Ok(tool_path) = which::which(tool) {
      // Canonicalize to resolve any symlinks (important for NixOS)
      let canonical_path =
        std::fs::canonicalize(&tool_path).unwrap_or(tool_path);
      // Only include if it's in /nix/store (which we mount) or is a direct binary
      let link = tools_dir.join(tool);
      #[cfg(unix)]
      std::os::unix::fs::symlink(&canonical_path, &link).map_err(|e| {
        CrylError::Sandbox(format!("Failed to symlink tool '{}': {}", tool, e))
      })?;
      #[cfg(not(unix))]
      std::fs::copy(&canonical_path, &link).map_err(|e| {
        CrylError::Sandbox(format!("Failed to copy tool '{}': {}", tool, e))
      })?;
    }
  }

  // Handle additional tools from args
  for tool in &sandbox.tools {
    if let Ok(tool_path) = which::which(tool) {
      // Canonicalize to resolve any symlinks (important for NixOS)
      let canonical_path =
        std::fs::canonicalize(&tool_path).unwrap_or(tool_path);
      let link = tools_dir.join(tool);
      if !link.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&canonical_path, &link).map_err(|e| {
          CrylError::Sandbox(format!(
            "Failed to symlink tool '{}': {}",
            tool, e
          ))
        })?;
        #[cfg(not(unix))]
        std::fs::copy(&canonical_path, &link).map_err(|e| {
          CrylError::Sandbox(format!("Failed to copy tool '{}': {}", tool, e))
        })?;
      }
    }
  }

  // Mount tools directory as read-only
  bwrap_args.push("--ro-bind".to_string());
  bwrap_args.push(tools_dir.to_string_lossy().to_string());
  bwrap_args.push("/tools".to_string());
  bwrap_args.push("--setenv".to_string());
  bwrap_args.push("PATH".to_string());
  bwrap_args.push("/tools".to_string());

  // Add the cryl command - use the original path if on NixOS, otherwise /bin/cryl
  let current_exe_str = current_exe.to_string_lossy().to_string();
  let cryl_path = if current_exe_str.starts_with("/nix/store/") {
    current_exe_str
  } else {
    "/bin/cryl".to_string()
  };

  bwrap_args.push("--".to_string());
  bwrap_args.push(cryl_path);
  bwrap_args.push("path".to_string());
  bwrap_args.push(sandbox_spec_target);

  // Add nosandbox flag (we're already in sandbox)
  bwrap_args.push("--nosandbox".to_string());

  // Add common args
  if common.dry_run {
    bwrap_args.push("--dry-run".to_string());
  }
  if common.allow_script {
    bwrap_args.push("--allow-script".to_string());
  }
  if common.verbose {
    bwrap_args.push("--verbose".to_string());
  }
  if common.very_verbose {
    bwrap_args.push("--very-verbose".to_string());
  }

  // Add max limits
  bwrap_args.push(format!("--max-imports={}", common.max_imports));
  bwrap_args.push(format!("--max-generations={}", common.max_generations));
  bwrap_args.push(format!("--max-exports={}", common.max_exports));
  bwrap_args.push(format!(
    "--max-specification-size={}",
    common.max_specification_size
  ));

  // Add manifest format
  bwrap_args.push(format!("--manifest-format={}", common.manifest_format));

  // Add no-manifest flag if set
  if common.no_manifest {
    bwrap_args.push("--no-manifest".to_string());
  }

  // Run bwrap
  let output =
    Command::new("bwrap")
      .args(&bwrap_args)
      .output()
      .map_err(|e| {
        CrylError::Sandbox(format!("Failed to execute bwrap: {}", e))
      })?;

  // Print stdout/stderr from sandboxed process
  if !output.stdout.is_empty() {
    eprint!("{}", String::from_utf8_lossy(&output.stdout));
  }
  if !output.stderr.is_empty() {
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
  }

  if !output.status.success() {
    return Err(CrylError::Sandbox(format!(
      "Sandboxed process failed with exit code: {:?}",
      output.status.code()
    )));
  }

  Ok(())
}

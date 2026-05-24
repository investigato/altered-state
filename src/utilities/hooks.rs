use crate::config::scenarios::{ScenarioHookConfig, ScenarioHookType};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
#[allow(dead_code)]
pub struct HookOutput {
    pub hook_type: ScenarioHookType,
    pub path: PathBuf,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}
pub fn execute_hooks(
    hooks: &[ScenarioHookConfig],
    hook_type: ScenarioHookType,
    base_dir: &Path,
) -> Result<Vec<HookOutput>, String> {
    let mut outputs = Vec::new();
    for hook in hooks {
        if hook.hook_type != hook_type {
            continue;
        }
        let script_path = if hook.path.is_absolute() {
            hook.path.clone()
        } else {
            base_dir.join(&hook.path)
        };
        if !script_path.exists() {
            return Err(format!("Hook script not found at path: {:?}", script_path));
        }
        let arguments = hook.arguments.clone();
        let continue_on_error = hook.continue_on_error;
        match execute_script_with_arguments(
            script_path.to_str().unwrap(),
            &arguments,
            continue_on_error,
        ) {
            Ok(_) => outputs.push(HookOutput {
                hook_type: hook.hook_type.clone(),
                path: script_path.clone(),
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
            Err(err) => {
                outputs.push(HookOutput {
                    hook_type: hook.hook_type.clone(),
                    path: script_path.clone(),
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: err.clone(),
                    success: false,
                });
                if !continue_on_error {
                    return Err(err);
                }
            }
        }
    }
    Ok(outputs)
}

pub fn execute_script_with_arguments(
    script_path: &str,
    arguments: &[String],
    continue_on_error: bool,
) -> Result<(), String> {
    let mut command = Command::new("powershell.exe");
    command.args([
        "-ExecutionPolicy",
        "Bypass",
        "-NonInteractive",
        "-File",
        script_path,
    ]);
    for arg in arguments {
        command.arg(arg);
    }
    let output = command
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        if continue_on_error {
            eprintln!(
                "Script execution failed with code {}: {}",
                output.status.code().unwrap_or(-1),
                error_message
            );
            Ok(())
        } else {
            Err(format!(
                "Script execution failed with code {}: {}",
                output.status.code().unwrap_or(-1),
                error_message
            ))
        }
    } else {
        println!("Script executed successfully");
        Ok(())
    }
}

use crate::objects::remediation::RemediationCommand;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
//     public static void WritePs1(List<RemediationCommand> commands, string cleanupScriptPath,
//         string cleanupScriptFile)
//     {
pub fn write_ps1(commands: &[RemediationCommand], cleanup_script_file: &Path) {
    //         var filePath = Path.Combine(cleanupScriptPath, cleanupScriptFile);
    let file_path = cleanup_script_file.to_path_buf();
    //         // make sure the damn file exists before we try to write to it
    //         if (!File.Exists(filePath))
    //         {
    //             using var fs = File.Create(filePath); //noaikido
    //             fs.Close();
    //         }
    if !file_path.exists() {
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directories for script path");
        }
        File::create(&file_path).expect("Failed to create script file");
    }
    //         using var writer = new StreamWriter(filePath);
    let mut writer = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_path)
        .expect("Failed to open script file for writing");
    //         writer.WriteLine("# AD Remediation Script");
    //         writer.WriteLine($"# Generated: {DateTime.Now}");
    //         writer.WriteLine();
    //         writer.WriteLine("Import-Module ActiveDirectory");
    //         writer.WriteLine();
    writer
        .write_all(b"# AD Remediation Script\n")
        .expect("Failed to write to script file");
    writer
        .write_all(format!("# Generated: {}\n\n", chrono::Local::now()).as_bytes())
        .expect("Failed to write to script file");
    writer
        .write_all(b"Import-Module ActiveDirectory\n\n")
        .expect("Failed to write to script file");
    //         foreach (var command in commands)
    //         {
    //             if (!string.IsNullOrEmpty(command.Description))
    //                 writer.WriteLine($"# {command.Description}");
    //             writer.WriteLine(command.IsComment ? $"# {command.Command}" : command.Command);
    //         }
    for command in commands {
        if let Some(description) = &command.description {
            writer
                .write_all(format!("# {}\n", description).as_bytes())
                .expect("Failed to write to script file");
        }
        let command_str = if command.is_comment {
            format!("# {}\n", command.command)
        } else {
            format!("{}\n", command.command)
        };
        writer
            .write_all(command_str.as_bytes())
            .expect("Failed to write to script file");
    }
    //         Console.WriteLine($"Script written to: {filePath}");
    //     }
    println!("Script written to: {}", file_path.to_string_lossy());
}
//     public static void WriteToConsole(List<RemediationCommand> commands)
//     {
#[allow(dead_code)]
pub fn write_to_console(commands: &[RemediationCommand]) {
    //
    //         Console.WriteLine("# AD Remediation Commands (dry run)");
    //         Console.WriteLine($"# Generated: {DateTime.Now}");
    //         Console.WriteLine();
    println!("# AD Remediation Commands (dry run)");
    println!("# Generated: {}\n", chrono::Local::now());
    //         foreach (var command in commands)
    //         {
    //             if (!string.IsNullOrEmpty(command.Description))
    //                 Console.WriteLine($"# {command.Description}");
    //             Console.WriteLine(command.IsComment ? $"# {command.Command}" : command.Command);
    //         }
    //     }
    for command in commands {
        if let Some(description) = &command.description {
            println!("# {}", description);
        }
        if command.is_comment {
            println!("# {}", command.command);
        } else {
            println!("{}", command.command);
        }
    }
}
//     public static void ExecuteScripts(string scriptPath, string psExecutable, Logger logger)
//     {
pub fn execute_script(script_path: &str) {
    //         logger.Info($"Executing script: {scriptPath} with PowerShell executable: {psExecutable}");
    println!("Executing script: {} with PowerShell", script_path);
    //
    //         var process = new Process
    //         {

    let mut process = Command::new("powershell.exe");
    //             StartInfo = new ProcessStartInfo
    //             {
    //                 FileName = psExecutable,
    //                 Arguments = $"-NonInteractive -File \"{scriptPath}\"",
    //                 UseShellExecute = false,
    //                 RedirectStandardError = true
    //             }
    //         };
    //         process.Start();
    //         process.WaitForExit();
    //         logger.Info($"Script exited with code: {process.ExitCode}");
    //         if (process.ExitCode != 0)
    //             logger.Error($"Script execution failed: {process.StandardError.ReadToEnd()}");
    //     }
    // }

    let output = process
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NonInteractive",
            "-File",
            script_path,
        ])
        .output()
        .expect("Failed to execute script");
    if output.status.success() {
        println!("Script executed successfully");
    } else {
        eprintln!(
            "Script execution failed with code {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

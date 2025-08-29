use crate::{PulseError, PulseResult};
use std::process::Command;

pub struct GitAdapter;

impl GitAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn get_changed_files(&self) -> PulseResult<Vec<String>> {
        println!("🔍 Getting changed files from git...");

        // Primero, obtener archivos modificados pero no staged
        let unstaged = Command::new("git")
            .args(["diff", "--name-only"])
            .output()
            .map_err(|e| {
                PulseError::fs_error(
                    format!("Failed to run git diff: {}", e),
                    Some("Ensure you're in a git repository and git is installed"),
                )
            })?;

        // Luego, obtener archivos staged
        let staged = Command::new("git")
            .args(["diff", "--name-only", "--cached"])
            .output()
            .map_err(|e| {
                PulseError::fs_error(
                    format!("Failed to run git diff --cached: {}", e),
                    Some("Ensure you're in a git repository and git is installed"),
                )
            })?;

        let mut files: Vec<String> = Vec::new();

        // Procesar archivos unstaged
        let unstaged_files = String::from_utf8(unstaged.stdout).map_err(|e| {
            PulseError::fs_error(
                format!("Invalid UTF-8 in git output: {}", e),
                Some("Check git configuration and file encoding"),
            )
        })?;
        files.extend(unstaged_files.lines().map(String::from));

        // Procesar archivos staged
        let staged_files = String::from_utf8(staged.stdout).map_err(|e| {
            PulseError::fs_error(
                format!("Invalid UTF-8 in git output: {}", e),
                Some("Check git configuration and file encoding"),
            )
        })?;
        files.extend(staged_files.lines().map(String::from));

        // Eliminar duplicados
        files.sort();
        files.dedup();

        if files.is_empty() {
            println!("   No changes detected in git");
        } else {
            println!("   Found {} changed files:", files.len());
            for file in &files {
                println!("   - {}", file);
            }
        }

        Ok(files)
    }

    pub fn is_in_repo() -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
use crate::domain::errors as domain_errors;
use crate::domain::ports::testing::ChangeDetectorPort;

impl ChangeDetectorPort for GitAdapter {
    fn changed_files(&self) -> domain_errors::PulseResult<Vec<String>> {
        self.get_changed_files()
            .map_err(|e| domain_errors::PulseError::Runtime(e.to_string()))
    }
}

//! Manages the command-line user interface, providing functions for printing banners,
//! status messages, progress bars, and formatted text to the console. It uses the
//! `colored`, `console`, and `indicatif` crates to enhance the visual presentation
//! of output.

use colored::*;
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::domain::ports::ui::{ProgressPort, UserInterfacePort};

/// A terminal user interface adapter that provides functions for displaying
/// formatted output, progress bars, and status messages in the console.
#[derive(Clone, Copy, Default)]
pub struct CliUiAdapter;

impl CliUiAdapter {
    /// Prints the application banner to the console.
    ///
    /// The banner is a stylized ASCII art representation of the application's name.
    pub fn print_banner() {
        let banner = r#"
    ██████╗ ██╗   ██╗██╗     ███████╗███████╗
    ██╔══██╗██║   ██║██║     ██╔════╝██╔════╝
    ██████╔╝██║   ██║██║     ███████╗█████╗  
    ██╔═══╝ ██║   ██║██║     ╚════██║██╔══╝  
    ██║     ╚██████╔╝███████╗███████║███████╗
    ╚═╝      ╚═════╝ ╚══════╝╚══════╝╚══════╝
    "#;

        println!("{}", banner.bright_cyan().bold());
        println!("{}", "    🚀".bright_white().bold());
        println!(
            "{}",
            "    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue()
        );
        println!();
    }

    /// Prints information about the current operating mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The current mode (e.g., "CI", "Development").
    /// * `dry_run` - A boolean indicating whether dry run mode is enabled.
    /// * `verbose` - A boolean indicating whether verbose mode is enabled.
    pub fn print_mode_info(mode: &str, dry_run: bool, verbose: bool) {
        let mode_color = match mode {
            "CI" => "red",
            "Development" => "green",
            "Debug" => "yellow",
            _ => "white",
        };

        println!(
            "  {} {}",
            "Mode:".bright_white().bold(),
            style(mode)
                .color256(match mode_color {
                    "red" => 196,
                    "green" => 46,
                    "yellow" => 226,
                    _ => 255,
                })
                .bold()
        );

        if dry_run {
            println!(
                "  {} {}",
                "🏃".bright_yellow(),
                "Dry run enabled - showing what would be executed".bright_yellow()
            );
        }

        if verbose {
            println!(
                "  {} {}",
                "🔍".bright_blue(),
                "Verbose mode enabled".bright_blue()
            );
        }
        println!();
    }

    /// Prints a section header with a title and an icon.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the section.
    /// * `icon` - An icon to display next to the title.
    pub fn print_section_header(title: &str, icon: &str) {
        println!("{} {}", icon.bright_cyan(), title.bright_white().bold());
        println!("{}", "─".repeat(50).bright_blue());
    }

    /// Prints a success message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    pub fn print_success(message: &str) {
        println!("{} {}", "✅".bright_green(), message.bright_green());
    }

    /// Prints an error message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    pub fn print_error(message: &str) {
        println!("{} {}", "❌".bright_red(), message.bright_red());
    }

    /// Prints a warning message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    pub fn print_warning(message: &str) {
        println!("{} {}", "⚠️".bright_yellow(), message.bright_yellow());
    }

    /// Prints an informational message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    pub fn print_info(message: &str) {
        println!("{} {}", "ℹ️".bright_blue(), message.bright_white());
    }

    /// Prints a debug message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    pub fn print_debug(message: &str) {
        println!("{} {}", "🐛".bright_magenta(), message.bright_black());
    }

    /// Creates a new progress bar.
    ///
    /// # Arguments
    ///
    /// * `len` - The total number of steps in the progress bar.
    /// * `message` - The message to display with the progress bar.
    ///
    /// # Returns
    ///
    /// A new `ProgressBar` instance.
    pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
                .expect("progress template")
                .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Prints the result of a single test.
    ///
    /// # Arguments
    ///
    /// * `spec` - The name of the test specification.
    /// * `status` - The status of the test (e.g., "PASS", "FAIL").
    /// * `duration` - The time it took to run the test.
    /// * `error` - An optional error message if the test failed.
    pub fn print_test_result(spec: &str, status: &str, duration: Duration, error: Option<&str>) {
        let (icon, color) = match status {
            "PASS" => ("✅", "green"),
            "FAIL" => ("❌", "red"),
            "SKIP" => ("⏭️", "yellow"),
            "FLAKY" => ("🔄", "magenta"),
            _ => ("⏳", "white"),
        };

        let duration_ms = duration.as_millis();
        let duration_str = if duration_ms > 1000 {
            format!("{:.1}s", duration.as_secs_f64())
        } else {
            format!("{}ms", duration_ms)
        };

        print!("  {} {} ", icon, spec.bright_white());
        print!(
            "({})",
            style(duration_str).color256(match color {
                "green" => 46,
                "red" => 196,
                "yellow" => 226,
                "magenta" => 201,
                _ => 255,
            })
        );

        if let Some(err) = error {
            println!();
            println!("    {}", err.bright_red());
        } else {
            println!();
        }
    }

    /// Prints a summary of test results.
    ///
    /// # Arguments
    ///
    /// * `total` - The total number of tests.
    /// * `passed` - The number of tests that passed.
    /// * `failed` - The number of tests that failed.
    /// * `skipped` - The number of tests that were skipped.
    /// * `duration` - The total time it took to run all tests.
    pub fn print_summary(
        total: usize,
        passed: usize,
        failed: usize,
        skipped: usize,
        duration: Duration,
    ) {
        println!();
        Self::print_section_header("Test Summary", "📊");

        let success_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "  {} {}",
            "Total Tests:".bright_white().bold(),
            total.to_string().bright_white()
        );
        println!(
            "  {} {}",
            "✅ Passed:".bright_green(),
            passed.to_string().bright_green()
        );

        if failed > 0 {
            println!(
                "  {} {}",
                "❌ Failed:".bright_red(),
                failed.to_string().bright_red()
            );
        }

        if skipped > 0 {
            println!(
                "  {} {}",
                "⏭️ Skipped:".bright_yellow(),
                skipped.to_string().bright_yellow()
            );
        }

        println!(
            "  {} {:.1}%",
            "Success Rate:".bright_white().bold(),
            style(format!("{:.1}%", success_rate)).color256(if success_rate >= 80.0 {
                46
            } else if success_rate >= 60.0 {
                226
            } else {
                196
            })
        );

        let duration_str = if duration.as_secs() > 60 {
            format!("{:.1}m", duration.as_secs_f64() / 60.0)
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        };

        println!(
            "  {} {}",
            "Duration:".bright_white().bold(),
            duration_str.bright_cyan()
        );
        println!();
    }

    /// Prints the status of watch mode.
    ///
    /// # Arguments
    ///
    /// * `enabled` - A boolean indicating whether watch mode is enabled.
    pub fn print_watch_status(enabled: bool) {
        if enabled {
            println!(
                "{} {}",
                "🔍".bright_green(),
                "Watch mode enabled - monitoring for changes...".bright_green()
            );
        } else {
            println!(
                "{} {}",
                "⏸️".bright_black(),
                "Watch mode disabled".bright_black()
            );
        }
    }

    /// Prints the status of the server.
    ///
    /// # Arguments
    ///
    /// * `status` - The current status of the server (e.g., "running", "stopped").
    /// * `url` - The URL of the server.
    pub fn print_server_status(status: &str, url: &str) {
        let (icon, color) = match status {
            "running" => ("🟢", "green"),
            "starting" => ("🟡", "yellow"),
            "stopped" => ("🔴", "red"),
            "error" => ("❌", "red"),
            _ => ("⚪", "white"),
        };

        println!(
            "  {} Server {} at {}",
            icon,
            style(status)
                .color256(match color {
                    "green" => 46,
                    "yellow" => 226,
                    "red" => 196,
                    _ => 255,
                })
                .bold(),
            url.bright_cyan()
        );
    }

    /// Prints information about the loaded configuration.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The path to the configuration file.
    /// * `valid` - A boolean indicating whether the configuration is valid.
    pub fn print_config_info(config_path: &str, valid: bool) {
        if valid {
            println!(
                "  {} Configuration loaded from {}",
                "✅".bright_green(),
                config_path.bright_cyan()
            );
        } else {
            println!(
                "  {} Invalid configuration at {}",
                "❌".bright_red(),
                config_path.bright_red()
            );
        }
    }

    /// Clears the terminal screen.
    pub fn clear_screen() {
        let term = Term::stdout();
        let _ = term.clear_screen();
    }

    /// Prints a hint about helpful commands.
    pub fn print_help_hint() {
        println!();
        println!(
            "{}",
            "💡 Consejo: Usa 'apicentric watch' para ejecutar pruebas al vuelo y 'apicentric status' para ver el simulador"
                .bright_blue()
                .italic()
        );
        println!("{}", "   Press Ctrl+C to stop watching".bright_black());
        println!();
    }

    /// Prints a message indicating that the TUI is launching.
    pub fn print_tui_launch() {
        Self::clear_screen();
        Self::print_banner();
        println!(
            "{}",
            "🎯 Preparando herramientas de línea de comandos..."
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "   Usa 'apicentric watch' para ejecutar al vuelo".bright_white()
        );
        println!("{}", "   Pulsa Ctrl+C para salir".bright_black());
        println!();

        // Brief pause for effect
        std::thread::sleep(Duration::from_millis(1000));
    }

    /// Prints a header for dry run mode.
    pub fn print_dry_run_header() {
        println!();
        println!("{}", "🏃 DRY RUN MODE".bright_yellow().bold());
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_yellow()
        );
        println!(
            "{}",
            "The following actions would be performed:".bright_white()
        );
        println!();
    }

    /// Prints a message about a file change.
    ///
    /// # Arguments
    ///
    /// * `file` - The path to the file that changed.
    /// * `change_type` - The type of change (e.g., "modified", "added", "deleted").
    pub fn print_file_change(file: &str, change_type: &str) {
        let (icon, color) = match change_type {
            "modified" => ("📝", "yellow"),
            "added" => ("➕", "green"),
            "deleted" => ("➖", "red"),
            _ => ("📄", "white"),
        };

        println!(
            "  {} {} {}",
            icon,
            style(change_type)
                .color256(match color {
                    "green" => 46,
                    "yellow" => 226,
                    "red" => 196,
                    _ => 255,
                })
                .bold(),
            file.bright_white()
        );
    }
}

/// A progress bar that can be used to show the progress of a long-running operation.
struct CliProgressBar {
    inner: ProgressBar,
}

impl ProgressPort for CliProgressBar {
    /// Increments the progress bar by a given amount.
    ///
    /// # Arguments
    ///
    /// * `delta` - The amount to increment the progress bar by.
    fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    /// Finishes the progress bar.
    fn finish(&self) {
        self.inner.finish();
    }
}

impl UserInterfacePort for CliUiAdapter {
    /// Prints a success message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    fn print_success(&self, message: &str) {
        CliUiAdapter::print_success(message);
    }

    /// Prints an error message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    fn print_error(&self, message: &str) {
        CliUiAdapter::print_error(message);
    }

    /// Prints a warning message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    fn print_warning(&self, message: &str) {
        CliUiAdapter::print_warning(message);
    }

    /// Prints an informational message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    fn print_info(&self, message: &str) {
        CliUiAdapter::print_info(message);
    }

    /// Prints a debug message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
    fn print_debug(&self, message: &str) {
        CliUiAdapter::print_debug(message);
    }

    /// Creates a new progress bar.
    ///
    /// # Arguments
    ///
    /// * `len` - The total number of steps in the progress bar.
    /// * `message` - The message to display with the progress bar.
    ///
    /// # Returns
    ///
    /// A new `ProgressBar` instance.
    fn create_progress_bar(&self, len: u64, message: &str) -> Box<dyn ProgressPort> {
        Box::new(CliProgressBar {
            inner: CliUiAdapter::create_progress_bar(len, message),
        })
    }
}

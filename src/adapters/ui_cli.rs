//! A command-line user interface adapter.
//!
//! This module provides a `CliUiAdapter` that implements the `UserInterfacePort`
//! trait. It is used to print messages and progress bars to the console.

use crate::domain::ports::ui::{ProgressPort, UserInterfacePort};

/// A progress bar for the command-line interface.
pub struct CliProgress {
    total: u64,
    message: String,
}

impl CliProgress {
    /// Creates a new `CliProgress`.
    ///
    /// # Arguments
    ///
    /// * `total` - The total number of steps in the progress bar.
    /// * `message` - The message to display with the progress bar.
    pub fn new(total: u64, message: &str) -> Self {
        println!("🔄 {} (0/{})", message, total);
        Self {
            total,
            message: message.to_string(),
        }
    }
}

impl ProgressPort for CliProgress {
    /// Increments the progress bar by a given amount.
    ///
    /// # Arguments
    ///
    /// * `_delta` - The amount to increment the progress bar by.
    fn inc(&self, _delta: u64) {
        // Keep output minimal to avoid flooding; could be enhanced to redraw
    }

    /// Finishes the progress bar.
    fn finish(&self) {
<<<<<<< HEAD
        println!(
            "✅ {} completed ({}/{})",
            self.message, self.total, self.total
        );
=======
        println!("✅ {} completed ({}/{})", self.message, self.total, self.total);
>>>>>>> origin/main
    }
}

/// A command-line user interface adapter.
pub struct CliUiAdapter;

impl UserInterfacePort for CliUiAdapter {
    /// Prints a success message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
<<<<<<< HEAD
    fn print_success(&self, message: &str) {
        println!("✅ {}", message);
    }
=======
    fn print_success(&self, message: &str) { println!("✅ {}", message); }
>>>>>>> origin/main
    /// Prints an error message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
<<<<<<< HEAD
    fn print_error(&self, message: &str) {
        println!("❌ {}", message);
    }
=======
    fn print_error(&self, message: &str) { println!("❌ {}", message); }
>>>>>>> origin/main
    /// Prints a warning message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
<<<<<<< HEAD
    fn print_warning(&self, message: &str) {
        println!("⚠️  {}", message);
    }
=======
    fn print_warning(&self, message: &str) { println!("⚠️  {}", message); }
>>>>>>> origin/main
    /// Prints an informational message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
<<<<<<< HEAD
    fn print_info(&self, message: &str) {
        println!("ℹ️  {}", message);
    }
=======
    fn print_info(&self, message: &str) { println!("ℹ️  {}", message); }
>>>>>>> origin/main
    /// Prints a debug message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to print.
<<<<<<< HEAD
    fn print_debug(&self, message: &str) {
        println!("🐛 {}", message);
    }
=======
    fn print_debug(&self, message: &str) { println!("🐛 {}", message); }
>>>>>>> origin/main

    /// Creates a new progress bar.
    ///
    /// # Arguments
    ///
    /// * `len` - The total number of steps in the progress bar.
    /// * `message` - The message to display with the progress bar.
    fn create_progress_bar(&self, len: u64, message: &str) -> Box<dyn ProgressPort> {
        Box::new(CliProgress::new(len, message))
    }
}

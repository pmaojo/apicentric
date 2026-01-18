//! User interface ports for displaying information to the user.

/// Represents a progress reporting handle returned by the user interface
/// when creating progress bars or similar visual indicators.
pub trait ProgressPort {
    /// Increment the progress by the given amount.
    fn inc(&self, delta: u64);

    /// Mark the progress operation as finished.
    fn finish(&self);
}

/// Abstraction over any kind of user interface (CLI, TUI, etc.) that can
/// display messages and show progress information to the user.
pub trait UserInterfacePort {
    fn print_success(&self, message: &str);
    fn print_error(&self, message: &str);
    fn print_warning(&self, message: &str);
    fn print_info(&self, message: &str);
    fn print_debug(&self, message: &str);

    /// Create a progress reporter for a task of the given length.
    fn create_progress_bar(&self, len: u64, message: &str) -> Box<dyn ProgressPort>;
}
<<<<<<< HEAD
=======

>>>>>>> origin/main

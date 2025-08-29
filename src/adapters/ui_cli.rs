use crate::domain::ports::ui::{ProgressPort, UserInterfacePort};

pub struct CliProgress {
    current: u64,
    total: u64,
    message: String,
}

impl CliProgress {
    pub fn new(total: u64, message: &str) -> Self {
        println!("🔄 {} (0/{})", message, total);
        Self {
            current: 0,
            total,
            message: message.to_string(),
        }
    }
}

impl ProgressPort for CliProgress {
    fn inc(&self, _delta: u64) {
        // Keep output minimal to avoid flooding; could be enhanced to redraw
    }

    fn finish(&self) {
        println!("✅ {} completed ({}/{})", self.message, self.total, self.total);
    }
}

pub struct CliUiAdapter;

impl UserInterfacePort for CliUiAdapter {
    fn print_success(&self, message: &str) { println!("✅ {}", message); }
    fn print_error(&self, message: &str) { println!("❌ {}", message); }
    fn print_warning(&self, message: &str) { println!("⚠️  {}", message); }
    fn print_info(&self, message: &str) { println!("ℹ️  {}", message); }
    fn print_debug(&self, message: &str) { println!("🐛 {}", message); }

    fn create_progress_bar(&self, len: u64, message: &str) -> Box<dyn ProgressPort> {
        Box::new(CliProgress::new(len, message))
    }
}


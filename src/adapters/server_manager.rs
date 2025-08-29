use crate::config::{ExecutionMode, ServerConfig};
use crate::{PulseError, PulseResult};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Port for server management operations
pub trait ServerManagerPort {
    fn check_server_health(&self, url: &str) -> PulseResult<bool>;
    fn start_server(&self, command: &str) -> PulseResult<ServerProcess>;
    fn wait_for_server(&self, url: &str, timeout_ms: u64) -> PulseResult<()>;
    fn should_check_server(&self, execution_mode: &ExecutionMode) -> bool;
}

/// Represents a running server process
pub struct ServerProcess {
    pub pid: u32,
    pub handle: Child,
}

impl ServerProcess {
    pub fn new(handle: Child) -> Self {
        let pid = handle.id();
        Self { pid, handle }
    }

    pub fn is_running(&mut self) -> bool {
        match self.handle.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking status, assume not running
        }
    }

    pub fn kill(&mut self) -> PulseResult<()> {
        self.handle.kill().map_err(|e| {
            PulseError::server_error(
                format!("Failed to kill server process {}: {}", self.pid, e),
                Some("The server process may have already exited"),
            )
        })
    }
}

/// Server manager implementation
///
/// Handles server lifecycle management including health checking, auto-start functionality,
/// and configurable retry logic. The server manager can be configured to:
///
/// - Check server health with configurable retries
/// - Auto-start development servers when needed
/// - Skip server checks in CI environments
/// - Handle different execution modes appropriately
pub struct ServerManager {
    config: ServerConfig,
}

impl ServerManager {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Check if server is responding at the given URL
    fn check_server_with_curl(&self, url: &str) -> PulseResult<bool> {
        let output = Command::new("curl")
            .arg("-s")
            .arg("-o")
            .arg("/dev/null")
            .arg("-w")
            .arg("%{http_code}")
            .arg("--connect-timeout")
            .arg("5")
            .arg("--max-time")
            .arg("10")
            .arg(url)
            .output();

        match output {
            Ok(result) => {
                let status_code = String::from_utf8_lossy(&result.stdout);
                let is_healthy = status_code.starts_with("2") || status_code.starts_with("3");
                Ok(is_healthy)
            }
            Err(_) => {
                // curl not available, try alternative method
                self.check_server_with_reqwest(url)
            }
        }
    }

    /// Fallback method using reqwest for server health check
    fn check_server_with_reqwest(&self, _url: &str) -> PulseResult<bool> {
        // For now, return false if curl is not available
        // In a real implementation, we could use reqwest or similar
        println!("⚠️ curl not available, skipping server health check");
        Ok(false)
    }

    /// Parse command string into command and arguments
    fn parse_command(&self, command_str: &str) -> (String, Vec<String>) {
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return (
                "npm".to_string(),
                vec!["run".to_string(), "dev".to_string()],
            );
        }

        let command = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        (command, args)
    }
}

impl ServerManagerPort for ServerManager {
    fn check_server_health(&self, url: &str) -> PulseResult<bool> {
        if self.config.skip_health_check {
            println!("🔄 Skipping server health check (disabled in config)");
            return Ok(true);
        }

        println!("🌐 Checking server health at {}...", url);

        let mut attempts = 0;
        let max_retries = self.config.health_check_retries;

        while attempts <= max_retries {
            match self.check_server_with_curl(url) {
                Ok(true) => {
                    println!("✅ Server is healthy at {}", url);
                    return Ok(true);
                }
                Ok(false) => {
                    if attempts < max_retries {
                        println!(
                            "⏳ Server not ready, retrying... ({}/{})",
                            attempts + 1,
                            max_retries
                        );
                        thread::sleep(Duration::from_millis(2000));
                    }
                }
                Err(e) => {
                    if attempts < max_retries {
                        println!(
                            "⚠️ Health check failed, retrying... ({}/{}): {}",
                            attempts + 1,
                            max_retries,
                            e
                        );
                        thread::sleep(Duration::from_millis(2000));
                    } else {
                        return Err(e);
                    }
                }
            }
            attempts += 1;
        }

        Ok(false)
    }

    fn start_server(&self, command: &str) -> PulseResult<ServerProcess> {
        println!("🚀 Starting server with command: {}", command);

        let (cmd, args) = self.parse_command(command);

        let child = Command::new(&cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                PulseError::server_error(
                    format!("Failed to start server with command '{}': {}", command, e),
                    Some("Check that the command is valid and all dependencies are installed"),
                )
            })?;

        let pid = child.id();
        println!("✅ Server started with PID: {}", pid);

        Ok(ServerProcess::new(child))
    }

    fn wait_for_server(&self, url: &str, timeout_ms: u64) -> PulseResult<()> {
        println!(
            "⏳ Waiting for server to be ready at {} (timeout: {}ms)...",
            url, timeout_ms
        );

        let start_time = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        while start_time.elapsed() < timeout {
            if self.check_server_health(url)? {
                println!("✅ Server is ready!");
                return Ok(());
            }

            thread::sleep(Duration::from_millis(1000));
        }

        Err(PulseError::server_error(
            format!("Server did not become ready within {}ms", timeout_ms),
            Some("Check server logs and ensure the server starts correctly"),
        ))
    }

    fn should_check_server(&self, execution_mode: &ExecutionMode) -> bool {
        match execution_mode {
            ExecutionMode::CI => false, // Skip server checks in CI
            ExecutionMode::Development | ExecutionMode::Debug => !self.config.skip_health_check,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;
    use crate::validation::ConfigValidator;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn create_test_server_config() -> ServerConfig {
        ServerConfig {
            auto_start: false,
            start_command: "echo test".to_string(),
            startup_timeout_ms: 5000,
            health_check_retries: 2,
            skip_health_check: false,
        }
    }

    #[test]
    fn test_server_manager_creation() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config.clone());
        assert_eq!(manager.config.start_command, config.start_command);
    }

    #[test]
    fn test_parse_command_simple() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        let (cmd, args) = manager.parse_command("npm run dev");
        assert_eq!(cmd, "npm");
        assert_eq!(args, vec!["run", "dev"]);
    }

    #[test]
    fn test_parse_command_complex() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        let (cmd, args) = manager.parse_command("node --inspect server.js --port 3000");
        assert_eq!(cmd, "node");
        assert_eq!(args, vec!["--inspect", "server.js", "--port", "3000"]);
    }

    #[test]
    fn test_parse_command_empty() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        let (cmd, args) = manager.parse_command("");
        assert_eq!(cmd, "npm");
        assert_eq!(args, vec!["run", "dev"]);
    }

    #[test]
    fn test_parse_command_single_word() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        let (cmd, args) = manager.parse_command("python");
        assert_eq!(cmd, "python");
        assert_eq!(args, Vec::<String>::new());
    }

    #[test]
    fn test_parse_command_with_quotes() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        // Note: This is a simple implementation that doesn't handle quotes
        // In a real implementation, we might want to use shell parsing
        let (cmd, args) = manager.parse_command("node server.js --name \"My App\"");
        assert_eq!(cmd, "node");
        assert_eq!(args, vec!["server.js", "--name", "\"My", "App\""]);
    }

    #[test]
    fn test_should_check_server_ci_mode() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        assert!(!manager.should_check_server(&ExecutionMode::CI));
    }

    #[test]
    fn test_should_check_server_development_mode() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        assert!(manager.should_check_server(&ExecutionMode::Development));
    }

    #[test]
    fn test_should_check_server_debug_mode() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        assert!(manager.should_check_server(&ExecutionMode::Debug));
    }

    #[test]
    fn test_should_check_server_skip_enabled() {
        let mut config = create_test_server_config();
        config.skip_health_check = true;
        let manager = ServerManager::new(config);

        assert!(!manager.should_check_server(&ExecutionMode::Development));
        assert!(!manager.should_check_server(&ExecutionMode::Debug));
    }

    #[test]
    fn test_server_process_creation() {
        // Create a dummy child process for testing
        let child = Command::new("echo")
            .arg("test")
            .spawn()
            .expect("Failed to spawn test process");

        let pid = child.id();
        let mut server_process = ServerProcess::new(child);

        assert_eq!(server_process.pid, pid);

        // Wait for the echo command to complete
        thread::sleep(Duration::from_millis(100));

        // Process should have exited
        assert!(!server_process.is_running());
    }

    #[test]
    fn test_server_process_long_running() {
        // Create a long-running process for testing
        let child = Command::new("sleep")
            .arg("1")
            .spawn()
            .expect("Failed to spawn test process");

        let mut server_process = ServerProcess::new(child);

        // Process should be running initially
        assert!(server_process.is_running());

        // Wait for process to complete
        thread::sleep(Duration::from_millis(1100));

        // Process should have exited
        assert!(!server_process.is_running());
    }

    #[test]
    fn test_server_process_kill() {
        // Create a long-running process
        let child = Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn test process");

        let mut server_process = ServerProcess::new(child);

        // Process should be running
        assert!(server_process.is_running());

        // Kill the process
        server_process.kill().expect("Failed to kill process");

        // Give it a moment to die
        thread::sleep(Duration::from_millis(100));

        // Process should no longer be running
        assert!(!server_process.is_running());
    }

    #[test]
    fn test_start_server_success() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        // Start a simple echo command
        let result = manager.start_server("echo hello");
        assert!(result.is_ok());

        let mut server_process = result.unwrap();
        assert!(server_process.pid > 0);

        // Wait for command to complete
        thread::sleep(Duration::from_millis(100));
        assert!(!server_process.is_running());
    }

    #[test]
    fn test_start_server_invalid_command() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        // Try to start a non-existent command
        let result = manager.start_server("nonexistent_command_12345");
        assert!(result.is_err());

        if let Err(PulseError::Server {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("Failed to start server"));
            assert!(suggestion.is_some());
            assert!(suggestion
                .unwrap()
                .contains("Check that the command is valid"));
        } else {
            panic!("Expected server error");
        }
    }

    #[test]
    fn test_check_server_health_skip_enabled() {
        let mut config = create_test_server_config();
        config.skip_health_check = true;
        let manager = ServerManager::new(config);

        // Should return true when health check is skipped
        let result = manager.check_server_health("http://localhost:9999");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_wait_for_server_timeout() {
        let config = create_test_server_config();
        let manager = ServerManager::new(config);

        // Try to wait for a server that will never be ready
        let result = manager.wait_for_server("http://localhost:9999", 100);
        assert!(result.is_err());

        if let Err(PulseError::Server {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("did not become ready"));
            assert!(suggestion.is_some());
        } else {
            panic!("Expected server error");
        }
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = create_test_server_config();

        // Test valid config
        assert!(config.validate().is_ok());

        // Test invalid timeout (too low)
        config.startup_timeout_ms = 1000;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "server.startup_timeout_ms"));

        // Reset and test invalid retry count
        config.startup_timeout_ms = 30000;
        config.health_check_retries = 0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "server.health_check_retries"));

        // Test empty start command
        config.health_check_retries = 5;
        config.start_command = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "server.start_command"));
    }

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();

        assert!(!config.auto_start);
        assert_eq!(config.start_command, "npm run dev");
        assert_eq!(config.startup_timeout_ms, 30000);
        assert_eq!(config.health_check_retries, 5);
        assert!(!config.skip_health_check);
    }

    // Mock server manager for testing without actual network calls
    #[cfg(test)]
    pub struct MockServerManager {
        pub health_responses: Arc<Mutex<HashMap<String, bool>>>,
        pub start_responses: Arc<Mutex<HashMap<String, Result<u32, String>>>>,
    }

    impl MockServerManager {
        pub fn new() -> Self {
            Self {
                health_responses: Arc::new(Mutex::new(HashMap::new())),
                start_responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn set_health_response(&self, url: &str, healthy: bool) {
            self.health_responses
                .lock()
                .unwrap()
                .insert(url.to_string(), healthy);
        }

        pub fn set_start_response(&self, command: &str, result: Result<u32, String>) {
            self.start_responses
                .lock()
                .unwrap()
                .insert(command.to_string(), result);
        }
    }

    impl ServerManagerPort for MockServerManager {
        fn check_server_health(&self, url: &str) -> PulseResult<bool> {
            let responses = self.health_responses.lock().unwrap();
            match responses.get(url) {
                Some(&healthy) => Ok(healthy),
                None => Ok(false), // Default to unhealthy
            }
        }

        fn start_server(&self, command: &str) -> PulseResult<ServerProcess> {
            let responses = self.start_responses.lock().unwrap();
            match responses.get(command) {
                Some(Ok(_pid)) => {
                    // Create a dummy process that immediately exits
                    let child = Command::new("echo").arg("mock").spawn().map_err(|e| {
                        PulseError::server_error(format!("Mock error: {}", e), None::<String>)
                    })?;
                    Ok(ServerProcess::new(child))
                }
                Some(Err(error)) => Err(PulseError::server_error(error.clone(), None::<String>)),
                None => Err(PulseError::server_error(
                    "Command not mocked",
                    None::<String>,
                )),
            }
        }

        fn wait_for_server(&self, url: &str, _timeout_ms: u64) -> PulseResult<()> {
            if self.check_server_health(url)? {
                Ok(())
            } else {
                Err(PulseError::server_error("Server not ready", None::<String>))
            }
        }

        fn should_check_server(&self, execution_mode: &ExecutionMode) -> bool {
            match execution_mode {
                ExecutionMode::CI => false,
                _ => true,
            }
        }
    }

    #[test]
    fn test_mock_server_manager() {
        let mock = MockServerManager::new();

        // Test health check
        mock.set_health_response("http://localhost:5173", true);
        assert!(mock.check_server_health("http://localhost:5173").unwrap());
        assert!(!mock.check_server_health("http://localhost:4000").unwrap());

        // Test server start
        mock.set_start_response("npm run dev", Ok(12345));
        let result = mock.start_server("npm run dev");
        assert!(result.is_ok());

        mock.set_start_response("invalid command", Err("Command failed".to_string()));
        let result = mock.start_server("invalid command");
        assert!(result.is_err());

        // Test wait for server
        mock.set_health_response("http://localhost:5173", true);
        assert!(mock.wait_for_server("http://localhost:5173", 1000).is_ok());

        mock.set_health_response("http://localhost:4000", false);
        assert!(mock.wait_for_server("http://localhost:4000", 1000).is_err());
    }
}

// Export MockServerManager for integration tests
#[cfg(test)]
pub use tests::MockServerManager;

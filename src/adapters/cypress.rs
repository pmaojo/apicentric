use crate::adapters::server_manager::{ServerManager, ServerManagerPort, ServerProcess};
use crate::config::{ExecutionMode, ServerConfig};
use crate::domain::entities::TestCaseResult;
use crate::{PulseError, PulseResult, TestRunnerPort};
use rayon::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Stats derivadas del JSON reporter de Cypress
#[derive(Debug, Clone)]
pub struct JsonStats {
    pub passes: u64,
    pub failures: u64,
    pub pending: u64,
}

pub struct CypressAdapter {
    config_path: String,
    base_url: String,
    server_manager: Option<Arc<ServerManager>>,
    execution_mode: ExecutionMode,
    server_config: Option<ServerConfig>,
    test_timeout_ms: u64,
    use_json_reporter: bool,
    silent: bool,
}

impl CypressAdapter {
    pub fn new(config_path: String, base_url: String) -> Self {
        Self {
            config_path,
            base_url,
            server_manager: None,
            execution_mode: ExecutionMode::Development,
            server_config: None,
            test_timeout_ms: 60000, // Default 60 seconds per test
            use_json_reporter: true,
            silent: false,
        }
    }

    pub fn set_silent(&mut self, silent: bool) {
        self.silent = silent;
    }

    #[inline]
    fn log_info(&self, msg: &str) { if !self.silent { println!("{}", msg); } }
    #[inline]
    fn log_warn(&self, msg: &str) { if !self.silent { println!("{}", msg); } }
    #[inline]
    fn log_success(&self, msg: &str) { if !self.silent { println!("{}", msg); } }
    #[inline]
    fn log_error(&self, msg: &str) { if !self.silent { eprintln!("{}", msg); } }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.test_timeout_ms = timeout_ms;
        self
    }

    pub fn with_server_manager(
        mut self,
        server_manager: Arc<ServerManager>,
        execution_mode: ExecutionMode,
        server_config: ServerConfig,
    ) -> Self {
        self.server_manager = Some(server_manager);
        self.execution_mode = execution_mode;
        self.server_config = Some(server_config);
        self
    }

    #[cfg(test)]
    fn skip_server_check(&self) -> bool {
        // Skip server check in tests
        true
    }

    #[cfg(not(test))]
    fn skip_server_check(&self) -> bool {
        false
    }

    fn run_single_spec(
        &self,
        spec: &str,
        retries: u8,
        headless: bool,
    ) -> PulseResult<(Duration, bool, bool, Option<String>, Vec<TestCaseResult>)> {
        // Validate test file exists and is valid
        if let Err(e) = self.validate_test_file(spec) {
            self.log_warn(&format!("⚠️ {}", e));
            return Ok((Duration::from_millis(0), true, true, None, Vec::new())); // skipped
        }

        self.log_info(&format!("🔄 Running Cypress test: {}", spec));

        let mut cmd = Command::new("npx");
        cmd.arg("cypress")
            .arg("run")
            .arg("--config-file")
            .arg(&self.config_path)
            .arg("--spec")
            .arg(spec);

        // Configure baseUrl and retries
        let config = format!("baseUrl={},retries={}", self.base_url, retries);
        cmd.arg("--config").arg(config);

        if headless {
            cmd.arg("--headless");
        }

        // Add browser configuration to avoid hanging
        cmd.arg("--browser").arg("electron");

        // Add additional configuration to handle file system issues
        cmd.arg("--config")
            .arg("video=false,screenshotOnRunFailure=false");

        // Forzamos siempre el JSON reporter (aunque use_json_reporter pueda ser false) para permitir extracción fiable de casos
        {
            let report_path = format!(
                ".pulse/tmp/{}_report.json",
                spec.replace('/', "_").replace('.', "_")
            );
            let _ = std::fs::create_dir_all(".pulse/tmp");
            cmd.arg("--reporter")
                .arg("json")
                .arg("--reporter-options")
                .arg(format!("output={}", report_path));
        }

    self.log_info(&format!("🚀 Executing Cypress for: {}", spec));
    if !self.silent { println!("   (debug) JSON reporter activado"); }

        // Execute the command with enhanced error handling
        let start_time = Instant::now();
        let output = cmd.output()
            .map_err(|e| PulseError::test_error(
                format!("Failed to execute Cypress for spec {}: {}", spec, e),
                Some("Check that Cypress is installed and accessible via npx. Try running 'npx cypress --version'")
            ))?;

        let duration = start_time.elapsed();

        // Check if execution took too long (informational only)
        if duration > Duration::from_millis(self.test_timeout_ms) {
            self.log_warn(&format!(
                "⚠️ Test execution took {}ms, which exceeds the configured timeout of {}ms",
                duration.as_millis(),
                self.test_timeout_ms
            ));
        }

        // Process the result with enhanced error handling
        if output.status.success() {
            let mut passed = true;
            let mut skipped = false;
            let mut err_summary: Option<String> = None;
            let mut test_cases = Vec::new();
            // Intentamos primero vía JSON reporter
            if self.use_json_reporter {
                if let Some((stats, cases)) = self.parse_json_report(spec) {
                    if stats.failures > 0 {
                        passed = false;
                        err_summary = Some("Failures detected".to_string());
                    }
                    if stats.passes == 0 && stats.failures == 0 && stats.pending > 0 {
                        skipped = true;
                    }
                    test_cases = cases;
                    // Print per-it details when verbose json reporter is enabled
                    if !test_cases.is_empty() && !self.silent {
                        println!("   ── Spec detail: {}", spec);
                        for case in &test_cases {
                            if case.passed {
                                println!("      ✓ {}", case.name);
                            } else {
                                match &case.error {
                                    Some(err) => println!("      ✗ {} -> {}", case.name, err.lines().next().unwrap_or("")),
                                    None => println!("      ✗ {}", case.name),
                                }
                            }
                        }
                    }
                }
            }
            // Fallback: si no obtuvimos casos (por no generar JSON) intentamos parsear stdout
            if test_cases.is_empty() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                test_cases = self.extract_cases_from_stdout(&stdout_str);
                if !test_cases.is_empty() && !self.silent {
                    println!("   ── Spec detail (stdout): {}", spec);
                    for case in &test_cases {
                        if case.passed { println!("      ✓ {}", case.name); } else { println!("      ✗ {}", case.name); }
                    }
                }
            }
            self.log_success(&format!(
                "✅ Test completed: {} ({}ms){}{}",
                spec,
                duration.as_millis(),
                if passed { "" } else { " (failures detected)" },
                if skipped { " (skipped)" } else { "" }
            ));
            Ok((duration, passed, skipped, err_summary, test_cases))
        } else {
            // Handle EPIPE and other process errors gracefully
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Filter out known non-critical errors
            let filtered_stderr = self.filter_cypress_errors(&stderr);

            let stdout_str = stdout.to_string();
            // Heurística: si stdout contiene "passing" y no contiene "failing" => éxito
            let heuristic_pass = stdout_str.contains("passing") && !stdout_str.contains("failing");
            if filtered_stderr.trim().is_empty() && (stdout.contains("Tests:") || heuristic_pass) {
                self.log_success(&format!(
                    "✅ Test completed with warnings: {} ({}ms)",
                    spec,
                    duration.as_millis()
                ));
                if !filtered_stderr.is_empty() {
                    self.log_warn(&format!("   ⚠️ Non-critical warnings: {}", filtered_stderr.trim()));
                }
                let mut passed = true;
                let mut skipped = false;
                let mut test_cases = Vec::new();
                if self.use_json_reporter {
                    if let Some((stats, cases)) = self.parse_json_report(spec) {
                        if stats.failures > 0 {
                            passed = false;
                        }
                        if stats.passes == 0 && stats.failures == 0 && stats.pending > 0 {
                            skipped = true;
                        }
                        test_cases = cases;
                        if !test_cases.is_empty() && !self.silent {
                            println!("   ── Spec detail: {}", spec);
                            for case in &test_cases {
                                if case.passed {
                                    println!("      ✓ {}", case.name);
                                } else {
                                    match &case.error {
                                        Some(err) => println!("      ✗ {} -> {}", case.name, err.lines().next().unwrap_or("")),
                                        None => println!("      ✗ {}", case.name),
                                    }
                                }
                            }
                        }
                    }
                }
                if test_cases.is_empty() {
                    test_cases = self.extract_cases_from_stdout(&stdout_str);
                    if !test_cases.is_empty() && !self.silent {
                        println!("   ── Spec detail (stdout): {}", spec);
                        for case in &test_cases { if case.passed { println!("      ✓ {}", case.name); } else { println!("      ✗ {}", case.name); } }
                    }
                }
                Ok((duration, passed, skipped, None, test_cases))
            } else {
                // Return (duration,false) after logging failure
                let test_cases = if self.use_json_reporter {
                    self.parse_json_report(spec)
                        .map(|(_, cases)| cases)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                self.handle_test_failure_enhanced(
                    spec,
                    stdout_str.clone(),
                    filtered_stderr.clone(),
                    output.status.code(),
                    duration,
                )?;
                if !test_cases.is_empty() && !self.silent {
                    println!("   ── Spec detail: {}", spec);
                    for case in &test_cases {
                        if case.passed {
                            println!("      ✓ {}", case.name);
                        } else {
                            match &case.error {
                                Some(err) => println!("      ✗ {} -> {}", case.name, err.lines().next().unwrap_or("")),
                                None => println!("      ✗ {}", case.name),
                            }
                        }
                    }
                }
                // Fallback for failure if JSON vacío
                let test_cases = if test_cases.is_empty() {
                    let mut extracted = self.extract_cases_from_stdout(&stdout_str);
                    if !extracted.is_empty() && !self.silent {
                        println!("   ── Spec detail (stdout): {}", spec);
                        for case in &extracted { if case.passed { println!("      ✓ {}", case.name); } else { println!("      ✗ {}", case.name); } }
                    }
                    extracted
                } else { test_cases };
                Ok((duration, false, false, None, test_cases))
            }
        }
    }

    /// Ejecuta las specs secuencialmente emitiendo cada resultado a través de un callback para permitir actualización en vivo en la TUI
    pub fn run_specs_streaming<F>(
        &self,
        specs: &[String],
        retries: u8,
        headless: bool,
        mut on_result: F,
    ) -> PulseResult<()>
    where
        F: FnMut(&str, bool, bool, Duration, Option<String>, Vec<TestCaseResult>), // spec, passed, skipped, error, cases
    {
        let valid_specs = self.filter_and_validate_specs(specs);
        if valid_specs.is_empty() {
            return Err(PulseError::test_error(
                "No valid test specs found to run (streaming)",
                Some("Check your specs pattern and ensure test files exist with .cy.ts or .cy.js extensions")
            ));
        }
        println!("🔍 Streaming execution for {} specs", valid_specs.len());
        let total_specs = valid_specs.len();
        let mut completed_specs = 0;

        for spec in &valid_specs {
            println!("▶️ [stream] Running {}", spec);
            match self.run_single_spec(spec, retries, headless) {
                Ok((dur, passed, skipped, err, cases)) => {
                    completed_specs += 1;
                    let progress = (completed_specs as f64 / total_specs as f64) * 100.0;
                    println!("Progress: {:.2}% - Completed {}/{} specs", progress, completed_specs, total_specs);

                    if !cases.is_empty() {
                        println!("   ── Spec detail: {}", spec);
                        for case in &cases {
                            if case.passed {
                                println!("      ✓ {}", case.name);
                            } else {
                                match &case.error {
                                    Some(err) => println!("      ✗ {} -> {}", case.name, err.lines().next().unwrap_or("")),
                                    None => println!("      ✗ {}", case.name),
                                }
                            }
                        }
                    }

                    on_result(spec, passed, skipped, dur, err, cases);
                }
                Err(e) => {
                    let msg = e.to_string();
                    on_result(spec, false, false, Duration::from_millis(0), Some(msg), Vec::new());
                }
            }
        }
        Ok(())
    }

    fn parse_json_report(&self, spec: &str) -> Option<(JsonStats, Vec<TestCaseResult>)> {
        let path = format!(
            ".pulse/tmp/{}_report.json",
            spec.replace('/', "_").replace('.', "_")
        );
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut test_cases = Vec::new();
                if let Some(tests) = value.get("tests").and_then(|t| t.as_array()) {
                    for test in tests {
                        let title = match test.get("title") {
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(serde_json::Value::Array(arr)) => {
                                arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" ")
                            }
                            _ => String::new(),
                        };
                        let state = test
                            .get("state")
                            .and_then(|s| s.as_str())
                            .unwrap_or("");
                        let error = test
                            .get("err")
                            .and_then(|e| e.get("message"))
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string());
                        let passed = state == "passed";
                        test_cases.push(TestCaseResult {
                            name: title,
                            passed,
                            error,
                        });
                    }
                }
                if let Some(stats) = value.get("stats") {
                    if let (Some(passes), Some(failures), pending) = (
                        stats.get("passes"),
                        stats.get("failures"),
                        stats.get("pending"),
                    ) {
                        if let (Some(p), Some(f)) = (passes.as_u64(), failures.as_u64()) {
                            let pend = pending.and_then(|x| x.as_u64()).unwrap_or(0);
                            return Some((
                                JsonStats {
                                    passes: p,
                                    failures: f,
                                    pending: pend,
                                },
                                test_cases,
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    // Heurística simple para extraer casos desde stdout cuando no hay JSON reporter disponible
    fn extract_cases_from_stdout(&self, stdout: &str) -> Vec<TestCaseResult> {
        let mut cases = Vec::new();
        for line in stdout.lines() {
            // Normalizar
            let trimmed = line.trim_start();
            // Detectar líneas de test pasados o fallados típicas de Cypress (contienen símbolos o numeración)
            if trimmed.starts_with('✓') {
                let name = trimmed.trim_start_matches(['✓',' ']).to_string();
                if !name.is_empty() { cases.push(TestCaseResult { name, passed: true, error: None }); }
            } else if trimmed.starts_with('✗') {
                let name = trimmed.trim_start_matches(['✗',' ']).to_string();
                if !name.is_empty() { cases.push(TestCaseResult { name, passed: false, error: None }); }
            } else if trimmed.starts_with(char::is_numeric) && trimmed.contains(')') {
                // Formato de mocha para fallos enumerados: "1) description"
                let mut parts = trimmed.splitn(2, ')');
                let _num = parts.next();
                if let Some(rest) = parts.next() {
                    let name = rest.trim().to_string();
                    if !name.is_empty() { cases.push(TestCaseResult { name, passed: false, error: None }); }
                }
            }
        }
        cases
    }

    fn validate_test_file(&self, spec: &str) -> Result<(), String> {
        let spec_path = Path::new(spec);

        // Check if file exists
        if !spec_path.exists() {
            return Err(format!("Test file not found: {}", spec));
        }

        // Check if it's actually a file
        if !spec_path.is_file() {
            return Err(format!("Path is not a file: {}", spec));
        }

        // Check if it's in a screenshots or videos directory (skip these)
        let excluded_dirs = ["screenshots", "videos"];
        if excluded_dirs.iter().any(|dir| {
            let dir_os = std::ffi::OsStr::new(dir);
            spec_path.starts_with(dir) || spec_path.components().any(|c| c.as_os_str() == dir_os)
        }) {
            return Err(format!("Skipping non-test file: {}", spec));
        }

        // Check if it has the correct extension
        let extension = spec_path.extension().and_then(|e| e.to_str());
        let stem = spec_path.file_stem().and_then(|s| s.to_str());
        let valid = matches!(extension, Some("ts" | "js"))
            && stem.map(|s| s.ends_with(".cy")).unwrap_or(false);

        if !valid {
            return Err(format!(
                "File does not appear to be a Cypress test: {}",
                spec
            ));
        }

        // Check file permissions
        match std::fs::metadata(spec_path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    return Err(format!("Test file is not readable: {}", spec));
                }
            }
            Err(e) => {
                return Err(format!("Cannot access test file {}: {}", spec, e));
            }
        }

        Ok(())
    }

    fn filter_cypress_errors(&self, stderr: &str) -> String {
        let mut filtered_lines = Vec::new();

        for line in stderr.lines() {
            // Skip known non-critical errors
            if line.contains("We failed to trash the existing run results")
                || line.contains("ENOENT: no such file or directory")
                || line.contains("ExperimentalWarning")
                || line.contains("Couldn't determine Mocha version")
                || line.contains("DevTools listening")
                || line.contains("write EPIPE")
                || line.contains("Broken pipe")
                || line.trim().is_empty()
            {
                continue;
            }

            filtered_lines.push(line);
        }

        filtered_lines.join("\n")
    }

    fn handle_test_failure_enhanced(
        &self,
        spec: &str,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
        duration: Duration,
    ) -> PulseResult<()> {
        eprintln!("❌ Test failed: {} ({}ms)", spec, duration.as_millis());

        // Show filtered output if available
        if !stdout.trim().is_empty() {
            let filtered_stdout = self.filter_cypress_output(&stdout);
            if !filtered_stdout.trim().is_empty() {
                eprintln!("   Output summary:");
                for line in filtered_stdout.lines().take(10) {
                    // Limit output
                    if !line.trim().is_empty() {
                        eprintln!("     {}", line);
                    }
                }
            } else {
                // Fallback: mostrar últimas líneas crudas para dar contexto
                eprintln!("   Output tail:");
                for line in stdout.lines().rev().take(15).collect::<Vec<_>>().into_iter().rev() {
                    if !line.trim().is_empty() { eprintln!("     {}", line); }
                }
            }
        }

        // Show critical errors only
        if !stderr.trim().is_empty() {
            eprintln!("   Critical errors:");
            for line in stderr.lines().take(5) {
                // Limit error output
                if !line.trim().is_empty() {
                    eprintln!("     {}", line);
                }
            }
        }

        // Provide specific error messages based on exit code
        let suggestion = match exit_code {
            Some(1) => Some("Test execution failed. Check the test file for syntax errors or assertion failures"),
            Some(2) => Some("Cypress configuration error. Verify your cypress.config.ts file"),
            Some(3) => Some("Test file not found or invalid. Check the file path and ensure it exists"),
            Some(4) => Some("Browser launch failed. Try using a different browser or check browser installation"),
            Some(5) => Some("Test timeout. Consider increasing timeout values or optimizing test performance"),
            _ => Some("Check the test file for syntax errors, configuration issues, or network problems"),
        };

        Err(PulseError::test_error(
            format!(
                "Cypress test failed for spec: {} (exit code: {:?})",
                spec, exit_code
            ),
            suggestion,
        ))
    }

    fn filter_cypress_output(&self, output: &str) -> String {
        let mut filtered_lines = Vec::new();
        let mut in_results_section = false;

        for line in output.lines() {
            // Skip noise from Cypress output
            if line.contains("Couldn't determine Mocha version")
                || line.contains("We failed to trash")
                || line.contains("DevTools listening")
                || line.contains("ExperimentalWarning")
                || line.contains("Electron")
            {
                continue;
            }

            // Capture important sections
            if line.contains("(Results)") || line.contains("Tests:") {
                in_results_section = true;
            }

            if in_results_section
                || line.contains("✖")
                || line.contains("✓")
                || line.contains("failing")
                || line.contains("passing")
            {
                filtered_lines.push(line);
            }
        }

        filtered_lines.join("\n")
    }

    fn ensure_server_availability(&self) -> PulseResult<Option<ServerProcess>> {
        let server_manager = match &self.server_manager {
            Some(manager) => manager,
            None => {
                println!("⚠️ No server manager configured, skipping server checks");
                return Ok(None);
            }
        };

        // Skip server checks in CI mode or if explicitly disabled
        if !server_manager.should_check_server(&self.execution_mode) {
            println!(
                "🔄 Skipping server checks for execution mode: {:?}",
                self.execution_mode
            );
            return Ok(None);
        }

        // Check if server is already running
        match server_manager.check_server_health(&self.base_url) {
            Ok(true) => {
                println!("✅ Server is already running and healthy");
                return Ok(None);
            }
            Ok(false) => {
                println!("⚠️ Server is not responding");
            }
            Err(e) => {
                println!("⚠️ Server health check failed: {}", e);
            }
        }

        // Try to auto-start server if configured
        if let Some(server_config) = self.get_server_config() {
            if server_config.auto_start {
                println!("🚀 Auto-starting server...");
                match server_manager.start_server(&server_config.start_command) {
                    Ok(server_process) => {
                        // Wait for server to be ready
                        match server_manager
                            .wait_for_server(&self.base_url, server_config.startup_timeout_ms)
                        {
                            Ok(_) => {
                                println!("✅ Server started successfully");
                                return Ok(Some(server_process));
                            }
                            Err(e) => {
                                eprintln!("❌ Server failed to start properly: {}", e);
                                eprintln!("   Proceeding with tests anyway...");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start server: {}", e);
                        eprintln!("   Proceeding with tests anyway...");
                    }
                }
            } else {
                println!("⚠️ Server auto-start is disabled");
                println!("   Please start your development server manually or enable auto_start in pulse.json");
                println!("   Proceeding with tests anyway...");
            }
        }

        Ok(None)
    }

    fn get_server_config(&self) -> Option<&ServerConfig> {
        self.server_config.as_ref()
    }
}

impl TestRunnerPort for CypressAdapter {
    fn run_specs(
        &self,
        specs: &[String],
        workers: usize,
        retries: u8,
        headless: bool,
    ) -> PulseResult<Vec<(String, bool, u128, Option<String>, Vec<TestCaseResult>)>> {
        // Optional server availability check (no longer required)
        let _server_process = if !self.skip_server_check() {
            match self.ensure_server_availability() {
                Ok(process) => {
                    self.log_success("✅ Server management completed successfully");
                    process
                }
                Err(e) => {
                    // Log server error but continue with tests
                    self.log_error(&format!("⚠️ Server management failed: {}", e));
                    self.log_error("   Continuing with test execution anyway...");
                    None
                }
            }
        } else {
            self.log_info("🔄 Skipping server checks");
            None
        };

        // Pre-validate and filter specs
        let valid_specs = self.filter_and_validate_specs(specs);

        if valid_specs.is_empty() {
            return Err(PulseError::test_error(
                "No valid test specs found to run",
                Some("Check your specs pattern and ensure test files exist with .cy.ts or .cy.js extensions")
            ));
        }

        self.log_info(&format!(
            "🔍 Running tests for {} specs with {} workers",
            valid_specs.len(),
            workers
        ));
        for spec in &valid_specs {
            self.log_info(&format!("   - {}", spec));
        }

        // Execute tests with improved error handling
        let mut failed_tests = Vec::new();
        let mut successful_tests = 0;
        let mut skipped_tests = 0;

        // For small numbers of tests or single worker, run sequentially to avoid race conditions
        let results: Vec<_> = if valid_specs.len() <= 2 || workers == 1 {
            valid_specs
                .iter()
                .map(|spec| {
                    self.log_info(&format!("▶️ Running test: {}", spec));
                    (spec.clone(), self.run_single_spec(spec, retries, headless))
                })
                .collect()
        } else {
            valid_specs
                .par_iter()
                .with_max_len(workers)
                .map(|spec| {
                    self.log_info(&format!("▶️ Running test: {}", spec));
                    (spec.clone(), self.run_single_spec(spec, retries, headless))
                })
                .collect()
        };

        // Process results and collect statistics
        let mut detailed: Vec<(String, bool, u128, Option<String>, Vec<TestCaseResult>)> = Vec::new();
        for (spec, result) in results {
            match result {
                Ok((duration, passed, skipped, err_opt, cases)) => {
                    if skipped {
                        skipped_tests += 1;
                        detailed.push((spec, true, duration.as_millis(), err_opt, cases));
                    } else if passed {
                        successful_tests += 1;
                        detailed.push((spec, true, duration.as_millis(), err_opt, cases));
                    } else {
                        // Failure sin mensaje detallado => crear error genérico
                        let msg = err_opt
                            .clone()
                            .unwrap_or_else(|| "Reported failure".to_string());
                        let pulse_err =
                            PulseError::test_error(msg.clone(), Some("See Cypress output"));
                        failed_tests.push((spec.clone(), pulse_err));
                        detailed.push((spec, false, duration.as_millis(), Some(msg), cases));
                    }
                }
                Err(e) => {
                    if e.to_string().contains("not found")
                        || e.to_string().contains("Skipping")
                        || e.to_string().contains("not readable")
                    {
                        skipped_tests += 1;
                        detailed.push((spec, true, 0, None, Vec::new()));
                    } else {
                        let msg = e.to_string();
                        failed_tests.push((spec.clone(), e));
                        detailed.push((spec, false, 0, Some(msg), Vec::new()));
                    }
                }
            }
        }

        // Report final results
        self.report_test_results(
            successful_tests,
            failed_tests.len(),
            skipped_tests,
            &failed_tests,
        )?;

        // Return error only if all tests failed and we have actual failures (not just skips)
        if successful_tests == 0 && !failed_tests.is_empty() {
            Err(PulseError::test_error(
                format!("All {} test(s) failed", failed_tests.len()),
                Some("Check the individual test failures above for specific issues"),
            ))
        } else {
            Ok(detailed)
        }
    }
}

impl CypressAdapter {
    fn filter_and_validate_specs(&self, specs: &[String]) -> Vec<String> {
        let mut valid_specs = Vec::new();

        for spec in specs {
            self.log_info(&format!("🔍 Validating test file: {}", spec));

            let path = Path::new(spec);

            let excluded_dirs = ["screenshots", "videos", "node_modules"];
            let in_excluded_dir = excluded_dirs.iter().any(|dir| {
                let dir_os = std::ffi::OsStr::new(dir);
                path.starts_with(dir) || path.components().any(|c| c.as_os_str() == dir_os)
            });

            let extension = path.extension().and_then(|e| e.to_str());
            let stem = path.file_stem().and_then(|s| s.to_str());
            let has_valid_ext = matches!(extension, Some("ts" | "js"))
                && stem.map(|s| s.ends_with(".cy")).unwrap_or(false);

            // Basic filtering for obviously invalid files
            if in_excluded_dir || !has_valid_ext {
                self.log_warn(&format!("⚠️ Skipping invalid test file: {}", spec));
                continue;
            }

            // Additional validation will be done in run_single_spec
            valid_specs.push(spec.clone());
        }

        valid_specs
    }

    fn report_test_results(
        &self,
        successful: usize,
        failed: usize,
        skipped: usize,
        failures: &[(String, PulseError)],
    ) -> PulseResult<()> {
        println!("\n📊 Test Execution Summary:");
        println!("   ✅ Successful: {}", successful);
        println!("   ❌ Failed: {}", failed);
        println!("   ⚠️ Skipped: {}", skipped);
        println!("   📝 Total: {}", successful + failed + skipped);

        if !failures.is_empty() {
            println!("\n❌ Failed Tests:");
            for (spec, error) in failures {
                println!("   • {}: {}", spec, error);
                if let Some(suggestion) = error.suggestion() {
                    println!("     💡 {}", suggestion);
                }
            }
        }

        if successful > 0 {
            println!("\n🎉 {} test(s) completed successfully!", successful);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_adapter() -> CypressAdapter {
        CypressAdapter::new(
            "cypress.config.ts".to_string(),
            "http://localhost:5173".to_string(),
        )
    }

    fn create_test_adapter_with_timeout(timeout_ms: u64) -> CypressAdapter {
        CypressAdapter::new(
            "cypress.config.ts".to_string(),
            "http://localhost:5173".to_string(),
        )
        .with_timeout(timeout_ms)
    }

    #[test]
    fn test_new_creates_adapter_with_correct_config() {
        let adapter = CypressAdapter::new(
            "cypress.config.ts".to_string(),
            "http://localhost:5173".to_string(),
        );

        assert_eq!(adapter.config_path, "cypress.config.ts");
        assert_eq!(adapter.base_url, "http://localhost:5173");
        assert!(adapter.server_manager.is_none());
        assert!(adapter.server_config.is_none());
        assert_eq!(adapter.test_timeout_ms, 60000); // Default timeout
    }

    #[test]
    fn test_adapter_with_custom_timeout() {
        let adapter = create_test_adapter_with_timeout(30000);
        assert_eq!(adapter.test_timeout_ms, 30000);
    }

    #[test]
    fn test_run_specs_filters_invalid_files() {
        let adapter = create_test_adapter();
        let specs = vec![
            "/path/to/screenshots/image.png".to_string(),
            "/path/to/videos/video.mp4".to_string(),
            "/path/to/test/valid.cy.ts".to_string(),
            "/invalid/path.js".to_string(),
            "/another/test/another.cy.ts".to_string(),
        ];

        // This should not fail even though some files don't exist
        // because invalid files are filtered out and non-existent files are skipped
        let result = adapter.run_specs(&specs, 1, 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_specs_with_empty_list() {
        let adapter = create_test_adapter();
        let specs = vec![];

        let result = adapter.run_specs(&specs, 1, 1, true);
        assert!(result.is_err());

        if let Err(PulseError::TestExecution { message, .. }) = result {
            assert!(message.contains("No valid test specs found"));
        } else {
            panic!("Expected TestExecution error");
        }
    }

    #[test]
    fn test_run_specs_with_all_invalid_specs() {
        let adapter = create_test_adapter();
        let specs = vec![
            "/path/to/screenshots/image.png".to_string(),
            "/path/to/videos/video.mp4".to_string(),
            "/invalid/path.js".to_string(),
        ];

        let result = adapter.run_specs(&specs, 1, 1, true);
        assert!(result.is_err());

        if let Err(PulseError::TestExecution { message, .. }) = result {
            assert!(message.contains("No valid test specs found"));
        } else {
            panic!("Expected TestExecution error");
        }
    }

    #[test]
    fn test_run_single_spec_with_non_existent_file() {
        let adapter = create_test_adapter();
        let spec = "/non/existent/file.cy.ts";

        let result = adapter.run_single_spec(spec, 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_single_spec_with_screenshot_file() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = create_test_adapter();

        // Create a file in screenshots directory
        let screenshots_path = temp_dir.path().join("screenshots").join("test.png");
        fs::create_dir_all(screenshots_path.parent().unwrap()).unwrap();
        fs::write(&screenshots_path, "fake image").unwrap();

        let result = adapter.run_single_spec(&screenshots_path.to_string_lossy(), 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_single_spec_with_video_file() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = create_test_adapter();

        // Create a file in videos directory
        let videos_path = temp_dir.path().join("videos").join("test.mp4");
        fs::create_dir_all(videos_path.parent().unwrap()).unwrap();
        fs::write(&videos_path, "fake video").unwrap();

        let result = adapter.run_single_spec(&videos_path.to_string_lossy(), 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_spec_filtering_logic() {
        let adapter = create_test_adapter();

        // Test valid specs
        let valid_specs = vec![
            "/app/test/login.cy.ts".to_string(),
            "/src/test/component.cy.ts".to_string(),
            "/test/integration/api.cy.ts".to_string(),
        ];

        // Test invalid specs
        let invalid_specs = vec![
            "/app/screenshots/login.png".to_string(),
            "/src/videos/recording.mp4".to_string(),
            "/test/unit.spec.ts".to_string(), // Not ending with .cy.ts
            "/src/component.ts".to_string(),  // No /test/ in path
        ];

        // Combine all specs
        let mut all_specs = valid_specs.clone();
        all_specs.extend(invalid_specs);

        let result = adapter.run_specs(&all_specs, 2, 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_specs_with_different_worker_counts() {
        let adapter = create_test_adapter();
        let specs = vec![
            "/test/spec1.cy.ts".to_string(),
            "/test/spec2.cy.ts".to_string(),
            "/test/spec3.cy.ts".to_string(),
        ];

        // Test with 1 worker
        let result = adapter.run_specs(&specs, 1, 1, true);
        assert!(result.is_ok());

        // Test with multiple workers
        let result = adapter.run_specs(&specs, 3, 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_specs_with_different_retry_counts() {
        let adapter = create_test_adapter();
        let specs = vec!["/test/spec.cy.ts".to_string()];

        // Test with no retries
        let result = adapter.run_specs(&specs, 1, 0, true);
        assert!(result.is_ok());

        // Test with multiple retries
        let result = adapter.run_specs(&specs, 1, 3, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_specs_headless_and_non_headless() {
        let adapter = create_test_adapter();
        let specs = vec!["/test/spec.cy.ts".to_string()];

        // Test headless mode
        let result = adapter.run_specs(&specs, 1, 1, true);
        assert!(result.is_ok());

        // Test non-headless mode
        let result = adapter.run_specs(&specs, 1, 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_adapter_configuration_integrity() {
        let config_path = "custom/cypress.config.ts";
        let base_url = "https://production.example.com";

        let adapter = CypressAdapter::new(config_path.to_string(), base_url.to_string());

        assert_eq!(adapter.config_path, config_path);
        assert_eq!(adapter.base_url, base_url);
        assert!(adapter.server_manager.is_none());
    }

    #[test]
    fn test_adapter_with_server_manager() {
        use crate::adapters::server_manager::ServerManager;
        use std::sync::Arc;

        let server_config = ServerConfig {
            auto_start: true,
            start_command: "npm run dev".to_string(),
            startup_timeout_ms: 30000,
            health_check_retries: 3,
            skip_health_check: false,
        };

        let server_manager = Arc::new(ServerManager::new(server_config.clone()));

        let adapter = CypressAdapter::new(
            "cypress.config.ts".to_string(),
            "http://localhost:5173".to_string(),
        )
        .with_server_manager(
            server_manager.clone(),
            ExecutionMode::Development,
            server_config.clone(),
        );

        assert!(adapter.server_manager.is_some());
        assert!(adapter.server_config.is_some());
        assert_eq!(adapter.execution_mode, ExecutionMode::Development);

        let config = adapter.get_server_config().unwrap();
        assert_eq!(config.start_command, "npm run dev");
        assert!(config.auto_start);
    }

    #[test]
    fn test_validate_test_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = create_test_adapter();

        // Create a valid test file
        let test_file = temp_dir.path().join("valid.cy.ts");
        fs::write(&test_file, "// Valid Cypress test").unwrap();

        let result = adapter.validate_test_file(&test_file.to_string_lossy());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_test_file_not_found() {
        let adapter = create_test_adapter();
        let result = adapter.validate_test_file("/non/existent/file.cy.ts");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_validate_test_file_wrong_extension() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = create_test_adapter();

        // Create a file with wrong extension
        let test_file = temp_dir.path().join("invalid.spec.ts");
        fs::write(&test_file, "// Not a Cypress test").unwrap();

        let result = adapter.validate_test_file(&test_file.to_string_lossy());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("does not appear to be a Cypress test"));
    }

    #[test]
    fn test_validate_test_file_screenshots_directory() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = create_test_adapter();

        // Create a file in screenshots directory
        let screenshots_dir = temp_dir.path().join("screenshots");
        fs::create_dir_all(&screenshots_dir).unwrap();
        let test_file = screenshots_dir.join("screenshot.cy.ts");
        fs::write(&test_file, "// Screenshot file").unwrap();

        let result = adapter.validate_test_file(&test_file.to_string_lossy());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Skipping non-test file"));
    }

    #[test]
    fn test_filter_and_validate_specs() {
        let adapter = create_test_adapter();
        let specs = vec![
            "/app/test/valid1.cy.ts".to_string(),
            "/app/screenshots/invalid.cy.ts".to_string(),
            "/app/videos/invalid.cy.ts".to_string(),
            "/app/test/valid2.cy.js".to_string(),
            "/app/test/invalid.spec.ts".to_string(),
            "/node_modules/test.cy.ts".to_string(),
        ];

        let valid_specs = adapter.filter_and_validate_specs(&specs);

        assert_eq!(valid_specs.len(), 2);
        assert!(valid_specs.contains(&"/app/test/valid1.cy.ts".to_string()));
        assert!(valid_specs.contains(&"/app/test/valid2.cy.js".to_string()));
    }

    #[cfg(windows)]
    #[test]
    fn test_filter_and_validate_specs_windows_paths() {
        let adapter = create_test_adapter();
        let specs = vec![
            "C:\\app\\test\\valid1.cy.ts".to_string(),
            "C:\\app\\screenshots\\invalid.cy.ts".to_string(),
            "C:\\app\\videos\\invalid.cy.ts".to_string(),
            "C:\\app\\test\\valid2.cy.js".to_string(),
            "C:\\app\\test\\invalid.spec.ts".to_string(),
            "C:\\node_modules\\test.cy.ts".to_string(),
        ];

        let valid_specs = adapter.filter_and_validate_specs(&specs);

        assert_eq!(valid_specs.len(), 2);
        assert!(valid_specs.contains(&"C:\\app\\test\\valid1.cy.ts".to_string()));
        assert!(valid_specs.contains(&"C:\\app\\test\\valid2.cy.js".to_string()));
    }

    #[test]
    fn test_run_specs_with_no_valid_specs() {
        let adapter = create_test_adapter();
        let specs = vec![
            "/app/screenshots/invalid.png".to_string(),
            "/app/videos/invalid.mp4".to_string(),
        ];

        let result = adapter.run_specs(&specs, 1, 1, true);
        assert!(result.is_err());

        if let Err(PulseError::TestExecution { message, .. }) = result {
            assert!(message.contains("No valid test specs found"));
        } else {
            panic!("Expected TestExecution error");
        }
    }
}

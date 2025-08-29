use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TestSpec {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub spec: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub test_cases: Vec<TestCaseResult>,
}

#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Copy, Clone)]
pub struct RetryPolicy {
    pub retries: u8,
}

#[derive(Debug, Clone)]
pub struct RouteMapping {
    pub route: String,
    pub specs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectPaths {
    pub routes_dir: String,
    pub specs_dir: String,
    pub reports_dir: String,
    pub index_cache_path: String,
}

#[derive(Debug, Clone)]
pub enum MetricsEvent {
    TestStart {
        spec: String,
    },
    TestEnd {
        spec: String,
        passed: bool,
        ms: u128,
    },
}

#[derive(Debug, Clone)]
pub struct ServerHealth {
    pub healthy: bool,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MockRoute {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
}

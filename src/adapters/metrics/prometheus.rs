use crate::{PulseError, PulseResult};
use hyper::service::service_fn;
use hyper::{body::Incoming, Request, Response};
use lazy_static::lazy_static;
use prometheus::{Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Registry, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref TESTS_EXECUTED: IntCounter = IntCounter::new(
        "pulse_tests_executed_total",
        "Total number of tests executed"
    )
    .unwrap();
    static ref TESTS_FAILED: IntCounter =
        IntCounter::new("pulse_tests_failed_total", "Total number of test failures").unwrap();
    static ref FLAKY_TESTS: IntGauge =
        IntGauge::new("pulse_flaky_tests_current", "Current number of flaky tests").unwrap();
    static ref TEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "pulse_test_duration_seconds",
            "Test execution duration in seconds"
        )
        .buckets(vec![1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
    )
    .unwrap();
    static ref TESTS_PASSED: IntCounter =
        IntCounter::new("pulse_tests_passed_total", "Total number of tests passed").unwrap();
    static ref TEST_SUITES_EXECUTED: IntCounter = IntCounter::new(
        "pulse_test_suites_executed_total",
        "Total number of test suites executed"
    )
    .unwrap();
}

pub struct PrometheusAdapter {
    _exporter_port: u16,
}

impl PrometheusAdapter {
    pub fn new(port: u16) -> PulseResult<Self> {
        // Register all metrics
        REGISTRY
            .register(Box::new(TESTS_EXECUTED.clone()))
            .map_err(|e| {
                PulseError::config_error(
                    format!("Failed to register tests_executed metric: {}", e),
                    None::<String>,
                )
            })?;
        REGISTRY
            .register(Box::new(TESTS_FAILED.clone()))
            .map_err(|e| {
                PulseError::config_error(
                    format!("Failed to register tests_failed metric: {}", e),
                    None::<String>,
                )
            })?;
        REGISTRY
            .register(Box::new(TESTS_PASSED.clone()))
            .map_err(|e| {
                PulseError::config_error(
                    format!("Failed to register tests_passed metric: {}", e),
                    None::<String>,
                )
            })?;
        REGISTRY
            .register(Box::new(FLAKY_TESTS.clone()))
            .map_err(|e| {
                PulseError::config_error(
                    format!("Failed to register flaky_tests metric: {}", e),
                    None::<String>,
                )
            })?;
        REGISTRY
            .register(Box::new(TEST_DURATION.clone()))
            .map_err(|e| {
                PulseError::config_error(
                    format!("Failed to register test_duration metric: {}", e),
                    None::<String>,
                )
            })?;
        REGISTRY
            .register(Box::new(TEST_SUITES_EXECUTED.clone()))
            .map_err(|e| {
                PulseError::config_error(
                    format!("Failed to register test_suites_executed metric: {}", e),
                    None::<String>,
                )
            })?;

        // Start HTTP server for metrics
        let addr: SocketAddr = ([0, 0, 0, 0], port).into();
        tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    println!(
                        "🔍 Prometheus metrics server started on http://0.0.0.0:{}/metrics",
                        port
                    );
                    listener
                }
                Err(e) => {
                    eprintln!(
                        "⚠️ Failed to bind Prometheus server to port {}: {}",
                        port, e
                    );
                    eprintln!("💡 Port {} is already in use. Try using a different port in your pulse.json configuration", port);
                    eprintln!("💡 Common alternative ports: 9091, 9092, 8080, 8090");
                    return;
                }
            };

            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = hyper_util::rt::TokioIo::new(stream);

                tokio::task::spawn(async move {
                    if let Err(err) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, service_fn(serve_metrics))
                        .await
                    {
                        eprintln!("❌ Prometheus metrics server error: {}", err);
                    }
                });
            }
        });

        Ok(Self {
            _exporter_port: port,
        })
    }

    pub fn record_test_execution(&self, duration: Duration, passed: bool) {
        TESTS_EXECUTED.inc();
        TEST_DURATION.observe(duration.as_secs_f64());

        if passed {
            TESTS_PASSED.inc();
        } else {
            TESTS_FAILED.inc();
        }
    }

    pub fn record_test_suite_execution(&self) {
        TEST_SUITES_EXECUTED.inc();
    }

    pub fn record_test_failure(&self) {
        TESTS_FAILED.inc();
    }

    pub fn record_test_success(&self) {
        TESTS_PASSED.inc();
    }

    pub fn update_flaky_tests(&self, count: i64) {
        FLAKY_TESTS.set(count);
    }

    pub fn get_metrics_summary(&self) -> String {
        format!(
            "Tests executed: {}, Passed: {}, Failed: {}, Flaky: {}",
            TESTS_EXECUTED.get(),
            TESTS_PASSED.get(),
            TESTS_FAILED.get(),
            FLAKY_TESTS.get()
        )
    }
}

async fn serve_metrics(_req: Request<Incoming>) -> Result<Response<String>, Infallible> {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    match encoder.encode(&REGISTRY.gather(), &mut buffer) {
        Ok(_) => {
            let response = String::from_utf8(buffer).unwrap_or_else(|e| {
                eprintln!("Error converting metrics to string: {}", e);
                String::from("# Error collecting metrics\n")
            });

            Ok(Response::builder()
                .status(200)
                .header("content-type", "text/plain; version=0.0.4")
                .body(response)
                .unwrap())
        }
        Err(e) => {
            eprintln!("Error encoding metrics: {}", e);
            Ok(Response::builder()
                .status(500)
                .body("# Error encoding metrics\n".to_string())
                .unwrap())
        }
    }
}

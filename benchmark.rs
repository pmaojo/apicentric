use std::time::Instant;
use apicentric::commands::gui::{GuiAppState, models::RequestLogEntry};

fn main() {
    let log_receiver = tokio::sync::broadcast::channel(1).1;
    let mut state = GuiAppState::new(log_receiver);
    
    // Benchmark adding 1000 logs
    let start = Instant::now();
    for i in 0..1000 {
        let entry = RequestLogEntry::new(
            "test-service".to_string(),
            "GET".to_string(),
            format!("/api/endpoint/{}", i),
            200,
            10,
        );
        state.add_request_log(entry);
    }
    let duration = start.elapsed();
    
    println!("Added 1000 logs in {:?}", duration);
    println!("Final log count: {}", state.request_log_count());
    
    // Benchmark filtering
    let start = Instant::now();
    let filtered = state.filtered_request_logs();
    let duration = start.elapsed();
    
    println!("Filtered {} logs in {:?}", filtered.len(), duration);
}

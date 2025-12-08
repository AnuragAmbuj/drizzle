use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "drizzle_http_requests_total",
        "Total number of HTTP requests",
        &["method", "status", "tenant_id"]
    )
    .unwrap();
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "drizzle_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "status"]
    )
    .unwrap();
}

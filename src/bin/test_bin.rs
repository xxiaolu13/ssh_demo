use metrics::histogram;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::thread;
use std::time::Duration;

fn main() {
    // 最简单的配置
    let builder = PrometheusBuilder::new();
    
    builder
        .with_http_listener(([0, 0, 0, 0], 9000))
        .install()
        .expect("failed to install recorder");

    println!("Server running on http://127.0.0.1:9000/metrics");

    loop {
        histogram!("test_histogram").record(42.0);
        println!("Recorded a sample...");
        thread::sleep(Duration::from_secs(1));
    }
}
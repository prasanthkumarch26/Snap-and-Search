use capture::capture_region;
use std::time::Instant;

fn main() {
    println!("Benchmarking Native Screen Capture (BitBlt + GetDIBits)...");

    // Warm up
    let _ = capture_region(0, 0, 100, 100);

    let resolutions = vec![
        ("1080p", 1920, 1080),
        ("1440p", 2560, 1440),
        ("4K", 3840, 2160),
    ];

    for (name, w, h) in resolutions {
        let mut times = vec![];
        for _ in 0..50 {
            let start = Instant::now();
            let _ = capture_region(0, 0, w, h).unwrap();
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50 = times[times.len() / 2];
        let p95 = times[(times.len() as f64 * 0.95) as usize];
        let p99 = times[(times.len() as f64 * 0.99) as usize];

        println!("{}: P50 = {:.2}ms, P95 = {:.2}ms, P99 = {:.2}ms", name, p50, p95, p99);
    }
}

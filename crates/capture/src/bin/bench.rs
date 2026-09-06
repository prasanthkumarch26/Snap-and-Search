use capture::CapturedFrame;
use std::time::Instant;

fn main() {
    let iterations = 100;
    println!("Starting benchmark with {} iterations...", iterations);

    let mut jpeg_times_1080 = Vec::new();
    let mut png_times_1080 = Vec::new();
    let mut jpeg_times_4k = Vec::new();
    let mut png_times_4k = Vec::new();

    let width_1080 = 1920;
    let height_1080 = 1080;
    
    let width_4k = 3840;
    let height_4k = 2160;

    // Generate random noise frames (worst case for compression)
    println!("Generating dummy frames...");
    let mut data_1080 = vec![0u8; width_1080 * height_1080 * 4];
    for (i, b) in data_1080.iter_mut().enumerate() { *b = (i % 255) as u8; }
    let frame_1080 = CapturedFrame { width: width_1080 as u32, height: height_1080 as u32, data: data_1080 };

    let mut data_4k = vec![0u8; width_4k * height_4k * 4];
    for (i, b) in data_4k.iter_mut().enumerate() { *b = (i % 255) as u8; }
    let frame_4k = CapturedFrame { width: width_4k as u32, height: height_4k as u32, data: data_4k };

    println!("Benchmarking 1080p...");
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = frame_1080.to_jpeg_bytes(90).unwrap();
        jpeg_times_1080.push(start.elapsed().as_micros() as u64);

        let start = Instant::now();
        let _ = frame_1080.to_png_bytes().unwrap();
        png_times_1080.push(start.elapsed().as_micros() as u64);
    }

    println!("Benchmarking 4K...");
    for _ in 0..50 { // 4k is slower, do fewer iterations
        let start = Instant::now();
        let _ = frame_4k.to_jpeg_bytes(90).unwrap();
        jpeg_times_4k.push(start.elapsed().as_micros() as u64);

        let start = Instant::now();
        let _ = frame_4k.to_png_bytes().unwrap();
        png_times_4k.push(start.elapsed().as_micros() as u64);
    }

    jpeg_times_1080.sort_unstable();
    png_times_1080.sort_unstable();
    jpeg_times_4k.sort_unstable();
    png_times_4k.sort_unstable();

    fn p(times: &[u64], percentile: f64) -> u64 {
        if times.is_empty() { return 0; }
        let idx = ((times.len() as f64) * percentile).round() as usize;
        let idx = idx.clamp(0, times.len() - 1);
        times[idx]
    }

    println!("\\n--- Benchmark Results ---");
    
    println!("\\nJPEG Encoding (Quality 90) - 1080p:");
    println!("  P50: {:.2} ms", p(&jpeg_times_1080, 0.50) as f64 / 1000.0);
    println!("  P95: {:.2} ms", p(&jpeg_times_1080, 0.95) as f64 / 1000.0);
    println!("  P99: {:.2} ms", p(&jpeg_times_1080, 0.99) as f64 / 1000.0);

    println!("\\nPNG Encoding (Lossless) - 1080p:");
    println!("  P50: {:.2} ms", p(&png_times_1080, 0.50) as f64 / 1000.0);
    println!("  P95: {:.2} ms", p(&png_times_1080, 0.95) as f64 / 1000.0);
    println!("  P99: {:.2} ms", p(&png_times_1080, 0.99) as f64 / 1000.0);

    println!("\\nJPEG Encoding (Quality 90) - 4K:");
    println!("  P50: {:.2} ms", p(&jpeg_times_4k, 0.50) as f64 / 1000.0);
    println!("  P95: {:.2} ms", p(&jpeg_times_4k, 0.95) as f64 / 1000.0);

    println!("\\nPNG Encoding (Lossless) - 4K:");
    println!("  P50: {:.2} ms", p(&png_times_4k, 0.50) as f64 / 1000.0);
    println!("  P95: {:.2} ms", p(&png_times_4k, 0.95) as f64 / 1000.0);
}

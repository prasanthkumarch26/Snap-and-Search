use capture::CapturedFrame;
use std::time::Instant;
use webp::Encoder;

fn main() {
    let width_1080 = 1920;
    let height_1080 = 1080;
    let mut data_1080 = vec![0u8; width_1080 * height_1080 * 4];
    for (i, b) in data_1080.iter_mut().enumerate() { *b = (i % 255) as u8; }
    let frame_1080 = CapturedFrame { width: width_1080 as u32, height: height_1080 as u32, data: data_1080 };

    println!("Benchmarking 1080p JPEG...");
    let start = Instant::now();
    for _ in 0..10 {
        let _ = frame_1080.to_jpeg_bytes(90).unwrap();
    }
    println!("JPEG avg: {:.2} ms", start.elapsed().as_secs_f64() * 1000.0 / 10.0);

    println!("Benchmarking 1080p WebP...");
    let start = Instant::now();
    for _ in 0..10 {
        // We have BGRA. The webp crate takes RGBA.
        // But for benchmark latency, it's roughly the same.
        let encoder = Encoder::from_rgba(&frame_1080.data, width_1080 as u32, height_1080 as u32);
        let _ = encoder.encode(90.0);
    }
    println!("WebP avg: {:.2} ms", start.elapsed().as_secs_f64() * 1000.0 / 10.0);
}

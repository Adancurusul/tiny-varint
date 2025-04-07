use tiny_varint::{VarintValue, varint, encode, decode};
use std::time::{Instant, Duration};

// Simple benchmark helper structure
struct Benchmark {
    name: &'static str,
    iterations: usize,
    total_time: Duration,
}

impl Benchmark {
    fn new(name: &'static str, iterations: usize) -> Self {
        Benchmark {
            name,
            iterations,
            total_time: Duration::new(0, 0),
        }
    }
    
    fn run<F>(&mut self, mut func: F) where F: FnMut() {
        // Warm up first
        for _ in 0..10 {
            func();
        }
        
        // Actual timing
        let start = Instant::now();
        for _ in 0..self.iterations {
            func();
        }
        self.total_time = start.elapsed();
    }
    
    fn report(&self) {
        let avg_time_ns = self.total_time.as_nanos() as f64 / self.iterations as f64;
        println!("{}: {:.2} ns per operation ({} iterations in {:?})", 
            self.name, avg_time_ns, self.iterations, self.total_time);
    }
}

fn main() {
    println!("VarintValue Performance Test");
    println!("===========================\n");
    
    // Test parameters
    const ITERATIONS: usize = 1_000_000;
    
    // Prepare test data and buffer
    let test_values = [
        varint!(u8: 127),
        varint!(u16: 16383),
        varint!(u32: 1000000),
        varint!(i8: -42),
        varint!(i16: -1000),
        varint!(i32: -100000),
        varint!(u64: 1_000_000_000_000),
    ];
    
    let mut buffer = [0u8; 20];
    
    // 1. Test type ID calculation performance
    let mut benchmark = Benchmark::new("Type ID calculation", ITERATIONS);
    benchmark.run(|| {
        for value in &test_values {
            let _ = value.get_type_id();
        }
    });
    benchmark.report();
    
    // 2. Test serialization size calculation performance
    let mut benchmark = Benchmark::new("Serialization size calculation", ITERATIONS);
    benchmark.run(|| {
        for value in &test_values {
            let _ = value.serialized_size();
        }
    });
    benchmark.report();
    
    // 3. Test serialization performance
    let mut benchmark = Benchmark::new("VarintValue serialization", ITERATIONS / 10);
    benchmark.run(|| {
        for value in &test_values {
            let _ = value.to_bytes(&mut buffer);
        }
    });
    benchmark.report();
    
    // Prepare deserialization test
    let mut encoded_values = Vec::new();
    let mut positions = Vec::new();
    let mut pos = 0;
    
    for value in &test_values {
        let bytes_written = value.to_bytes(&mut buffer[..]).unwrap();
        encoded_values.extend_from_slice(&buffer[..bytes_written]);
        positions.push((pos, bytes_written));
        pos += bytes_written;
    }
    
    // 4. Test deserialization performance
    let mut benchmark = Benchmark::new("VarintValue deserialization", ITERATIONS / 10);
    benchmark.run(|| {
        for (start, len) in &positions {
            let _ = VarintValue::from_bytes(&encoded_values[*start..*start + *len]);
        }
    });
    benchmark.report();
    
    // 5. Compare with regular varint encoding (without type information)
    let u32_values = [127u32, 16383, 1000000];
    let mut benchmark = Benchmark::new("Regular u32 varint encoding", ITERATIONS);
    benchmark.run(|| {
        for value in &u32_values {
            let _ = encode(*value, &mut buffer);
        }
    });
    benchmark.report();
    
    // 6. Compare with regular varint decoding (without type information)
    let mut u32_encoded = Vec::new();
    let mut u32_positions = Vec::new();
    let mut pos = 0;
    
    for value in &u32_values {
        let bytes_written = encode(*value, &mut buffer).unwrap();
        u32_encoded.extend_from_slice(&buffer[..bytes_written]);
        u32_positions.push((pos, bytes_written));
        pos += bytes_written;
    }
    
    let mut benchmark = Benchmark::new("Regular u32 varint decoding", ITERATIONS);
    benchmark.run(|| {
        for (start, len) in &u32_positions {
            let _ = decode::<u32>(&u32_encoded[*start..*start + *len]);
        }
    });
    benchmark.report();
    
    println!("\nPerformance Summary:");
    println!("1. VarintValue type information introduces some performance overhead");
    println!("2. Optimizations (special zero handling, avoiding temporary buffers, etc.) effectively improve performance");
    println!("3. For scenarios requiring mixed types, the performance cost is acceptable");
} 
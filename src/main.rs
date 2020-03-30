#![feature(exclusive_range_pattern)]
#![feature(test)]
extern crate test;

use std::time::{Duration, SystemTime};

mod block_hex;
mod block_json;
mod fixed_hash;

use block_hex::StructFromBytes;

static BLOCK_JSON: &str = include_str!("../blocks/623200.json");
static BLOCK_HEX: &str = include_str!("../blocks/623200.hex");

fn main() {
    measure("JSON", 100, || {
        let _block: block_json::Block = serde_json::from_str(BLOCK_JSON).unwrap();
    });

    let data = BLOCK_HEX.trim();
    measure("HEX", 100, || {
        let bytes = hex::decode(data).unwrap();
        let _block = block_hex::Block::from_bytes(&mut &bytes[..]);
    });

    let data = BLOCK_HEX.trim();
    measure("HEX2", 100, || {
        let bytes = hex_decode(data);
        let _block = block_hex::Block::from_bytes(&mut &bytes[..]);
    });
}

fn hex_decode_symbol(sym: u8) -> u8 {
    match sym {
        b'0'..=b'9' => sym - b'0',
        b'a'..=b'f' => sym - b'a' + 10,
        _ => panic!("Not hex symbol: {}", String::from_utf8_lossy(&[sym])),
    }
}

fn hex_decode(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|pair| (hex_decode_symbol(pair[0]) << 4) + hex_decode_symbol(pair[1]))
        .collect::<Vec<u8>>()
}

fn measure<F>(text: &str, iters: usize, fn_test: F)
where
    F: Fn(),
{
    let mut elapsed = Vec::with_capacity(iters);
    for _ in 0..iters {
        let ts = SystemTime::now();
        fn_test();
        elapsed.push(ts.elapsed().unwrap().as_nanos() as u64);
    }

    let average: u64 = elapsed.iter().sum::<u64>() / iters as u64;
    elapsed.sort();

    println!("Parse {} ({} iterations):", text, iters);
    println!("min: {:?}", Duration::from_nanos(*elapsed.first().unwrap()));
    println!("average: {:?}", Duration::from_nanos(average));
    println!("max: {:?}", Duration::from_nanos(*elapsed.last().unwrap()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn parse_json(b: &mut Bencher) {
        b.iter(|| {
            let _block: block_json::Block = serde_json::from_str(BLOCK_JSON).unwrap();
        });
    }

    #[bench]
    fn parse_hex(b: &mut Bencher) {
        let data = BLOCK_HEX.trim();
        b.iter(|| {
            let bytes = hex::decode(data).unwrap();
            let _block = block_hex::Block::from_bytes(&mut &bytes[..]);
        });
    }

    #[bench]
    fn parse_hex2(b: &mut Bencher) {
        let data = BLOCK_HEX.trim();
        b.iter(|| {
            let bytes = hex_decode(data);
            let _block = block_hex::Block::from_bytes(&mut &bytes[..]);
        });
    }
}

#![no_main]

use libfuzzer_sys::fuzz_target;
use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

const MAX_INPUT_SIZE: usize = 16 * 1024;

fn read_u64(bytes: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    let len = bytes.len().min(8);
    arr[..len].copy_from_slice(&bytes[..len]);
    u64::from_le_bytes(arr)
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }

    let decimals = (data[0] % 10) as u32;
    let numer = read_u64(data.get(1..9).unwrap_or(&[])) as usize;
    let denom = read_u64(data.get(9..17).unwrap_or(&[])) as usize;

    let value_bits = read_u64(data.get(17..25).unwrap_or(&[]));
    let value = f64::from_bits(value_bits);
    if !value.is_finite() {
        return;
    }

    let rounded_once = round_f64(value, decimals);
    let rounded_twice = round_f64(rounded_once, decimals);
    assert_eq!(rounded_once, rounded_twice);

    if denom == 0 {
        assert_eq!(safe_ratio(numer, denom), 0.0);
    }

    if numer > 0 {
        assert_eq!(safe_ratio(numer, numer), 1.0);
    }

    let mut values: Vec<usize> = data
        .get(25..)
        .unwrap_or(&[])
        .iter()
        .map(|b| *b as usize)
        .collect();
    if values.is_empty() {
        values.push(0);
    }
    values.sort_unstable();

    let pct = (data[0] as f64) / 255.0;
    let p = percentile(&values, pct);
    assert!(p >= *values.first().unwrap() as f64);
    assert!(p <= *values.last().unwrap() as f64);
    assert_eq!(p, percentile(&values, pct));

    let gini = gini_coefficient(&values);
    assert!(gini >= 0.0);
    assert!(gini <= 1.0);
    assert_eq!(gini, gini_coefficient(&values));
});

use std::env;
use num_bigint::BigInt;
use num_traits::{Zero, Num}; // <-- Num is required for from_str_radix

/// Entry point
fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: cargo run <hexone> <hextwo> <OP>");
        return;
    }

    let hexone = &args[1];
    let hextwo = &args[2];
    let op = &args[3].to_uppercase();

    // Convert hex strings to BigInt
    let x = parse_hex(hexone);
    let y = parse_hex(hextwo);

    // Choose the operation
    let result = match op.as_str() {
        "ADD" => add_ix(&x, &y),
        "SUB" => sub_ix(&x, &y),
        "MUL" => mul_ix(&x, &y),
        "QUO" => div_ix(&x, &y),
        "REM" => rem_ix(&x, &y),
        _ => {
            eprintln!("Unknown operation: {}", op);
            return;
        }
    };

    // Print as lowercase hex (with 0x prefix)
    println!("{:#x}", result);
}

/// Convert hex string (like 0x123abc) to BigInt
fn parse_hex(s: &str) -> BigInt {
    let s = s.trim_start_matches("0x");
    BigInt::from_str_radix(s, 16).expect("Invalid hex")
}

/// Addition
fn add_ix(a: &BigInt, b: &BigInt) -> BigInt {
    a + b
}

/// Subtraction
fn sub_ix(a: &BigInt, b: &BigInt) -> BigInt {
    a - b
}

/// Multiplication
fn mul_ix(a: &BigInt, b: &BigInt) -> BigInt {
    a * b
}

/// Division (integer division)
fn div_ix(a: &BigInt, b: &BigInt) -> BigInt {
    if b.is_zero() {
        panic!("Division by zero");
    }
    a / b
}

/// Remainder (modulus)
fn rem_ix(a: &BigInt, b: &BigInt) -> BigInt {
    if b.is_zero() {
        panic!("Remainder by zero");
    }
    a % b
}

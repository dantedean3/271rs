// =========================================================
// File: main.rs
// Author: D’Ante Dean
// Course: CS 271 — Traits & Systems in Rust
// =========================================================

use traits_systems_92::*;

// ---------- Stack Tests ----------
fn test_stack() {
    println!("=== Stack Tests ===");
    let mut s: Stack<String> = stack();
    let mut popped: Option<String>;

    s = s.push(String::from("0"));
    s = s.push(String::from("1"));
    (popped, s) = s.pop();
    println!("Stack POP: should be Some(\"1\"), is {popped:?}");
    s = s.push(String::from("n"));
    (popped, s) = s.pop();
    println!("Stack POP: should be Some(\"n\"), is {popped:?}");
    (popped, s) = s.pop();
    println!("Stack POP: should be Some(\"0\"), is {popped:?}");
    (popped, s) = s.pop();
    println!("Stack POP: should be None, is {popped:?}");
    s = s.push(String::from("m"));
    (popped, _) = s.pop();
    println!("Stack POP: should be Some(\"m\"), is {popped:?}");

    let mut s: Stack<i32> = stack();
    let popped: Option<i32>;
    s = s.push(123);
    s = s.push(456);
    (popped, _) = s.pop();
    println!("Stack POP: should be Some(456), is {popped:?}");
}

// ---------- Queue Tests ----------
fn test_queue() {
    println!("=== Queue Tests ===");
    let mut s: Queue<String> = queue();
    let mut popped: Option<String>;

    s = s.push(String::from("0"));
    s = s.push(String::from("1"));
    (popped, s) = s.pop();
    println!("Queue POP: should be Some(\"0\"), is {popped:?}");
    s = s.push(String::from("n"));
    (popped, s) = s.pop();
    println!("Queue POP: should be Some(\"1\"), is {popped:?}");
    (popped, s) = s.pop();
    println!("Queue POP: should be Some(\"n\"), is {popped:?}");
    (popped, s) = s.pop();
    println!("Queue POP: should be None, is {popped:?}");
    s = s.push(String::from("m"));
    (popped, _) = s.pop();
    println!("Queue POP: should be Some(\"m\"), is {popped:?}");

    let mut s: Queue<i32> = queue();
    let popped: Option<i32>;
    s = s.push(123);
    s = s.push(456);
    (popped, _) = s.pop();
    println!("Queue POP: should be Some(123), is {popped:?}");
}

fn main() {
    test_stack();
    test_queue();
}

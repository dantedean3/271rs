// =========================================================
// File: lib.rs
// Author: D’Ante Dean
// Course: CS 271 — Traits & Systems in Rust
// Date: November 7 2025
// =========================================================

// ===== TRAITS =====

// Defines a push operation
pub trait Push<T> {
    fn push(self, val: T) -> Self;
}

// Defines a pop operation
pub trait Pop<T> {
    fn pop(self) -> (Option<T>, Self);
}

// ===== STACK =====
// Stack: Last In, First Out (LIFO)
#[derive(Debug, Clone)]
pub struct Stack<T> {
    data: Vec<T>,
}

// Create an empty stack
pub fn stack<T>() -> Stack<T> {
    Stack { data: Vec::new() }
}

// Implement push for Stack
impl<T> Push<T> for Stack<T> {
    fn push(mut self, val: T) -> Self {
        self.data.push(val);
        self
    }
}

// Implement pop for Stack
impl<T> Pop<T> for Stack<T> {
    fn pop(mut self) -> (Option<T>, Self) {
        let popped = self.data.pop();
        (popped, self)
    }
}

// ===== QUEUE =====
// Queue: First In, First Out (FIFO)
#[derive(Debug, Clone)]
pub struct Queue<T> {
    data: Vec<T>,
}

// Create an empty queue
pub fn queue<T>() -> Queue<T> {
    Queue { data: Vec::new() }
}

// Implement push for Queue
impl<T> Push<T> for Queue<T> {
    fn push(mut self, val: T) -> Self {
        self.data.push(val);
        self
    }
}

// Implement pop for Queue
impl<T> Pop<T> for Queue<T> {
    fn pop(mut self) -> (Option<T>, Self) {
        if self.data.is_empty() {
            (None, self)
        } else {
            let popped = Some(self.data.remove(0));
            (popped, self)
        }
    }
}

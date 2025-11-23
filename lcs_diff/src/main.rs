use std::env;
use std::fs;

// ============================================================
// main()
// ============================================================

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run <file1> <file2>");
        return;
    }

    let left = fname_to_lines(&args[1]);
    let right = fname_to_lines(&args[2]);

    let dp = build_lcs_table(&left, &right);
    let (ops, match_positions) = backtrack(&left, &right, &dp);
    let blocks = group_into_blocks(ops, match_positions);

    for b in blocks {
        println!("{}", b);
    }
}

// ============================================================
// Read file → Vec<String>
// ============================================================

fn fname_to_lines(fname: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let text = fs::read_to_string(fname).expect("Failed to read file");
    for line in text.lines() {
        lines.push(line.to_string());
    }
    lines
}

// ============================================================
// Build LCS DP table
// ============================================================

fn build_lcs_table(left: &Vec<String>, right: &Vec<String>) -> Vec<Vec<usize>> {
    let m = left.len();
    let n = right.len();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if left[i - 1] == right[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

#[derive(Debug)]
enum Op {
    Delete(usize, String),
    Add(usize, String),
}

// ============================================================
// Backtrack DP and record matches too
// ============================================================

fn backtrack(
    left: &Vec<String>,
    right: &Vec<String>,
    dp: &Vec<Vec<usize>>
) -> (Vec<Op>, Vec<usize>) {

    let mut ops = Vec::new();
    let mut matches = Vec::new(); // <-- exact match line numbers

    let mut i = left.len();
    let mut j = right.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && left[i - 1] == right[j - 1] {
            matches.push(i); // <-- matched at left line i
            i -= 1;
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i - 1][j] >= dp[i][j - 1]) {
            ops.push(Op::Delete(i, left[i - 1].clone()));
            i -= 1;
        } else {
            ops.push(Op::Add(j, right[j - 1].clone()));
            j -= 1;
        }
    }

    ops.reverse();
    matches.reverse();
    (ops, matches)
}

// ============================================================
// GROUP BLOCKS – correct diff behavior
// ============================================================

fn group_into_blocks(ops: Vec<Op>, matches: Vec<usize>) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut i = 0;

    let mut match_index = 0;
    let mut last_match_line = if matches.is_empty() { 0 } else { matches[0] };

    while i < ops.len() {
        let mut del_nums = Vec::new();
        let mut add_nums = Vec::new();
        let mut del_lines = Vec::new();
        let mut add_lines = Vec::new();

        // Start block
        match &ops[i] {
            Op::Delete(n, s) => {
                del_nums.push(*n);
                del_lines.push(format!("< {}", s));
            }
            Op::Add(n, s) => {
                add_nums.push(*n);
                add_lines.push(format!("> {}", s));
            }
        }

        i += 1;

        // Consume operations into block
        while i < ops.len() {
            match &ops[i] {
                Op::Delete(n, s) => {
                    del_nums.push(*n);
                    del_lines.push(format!("< {}", s));
                }
                Op::Add(n, s) => {
                    add_nums.push(*n);
                    add_lines.push(format!("> {}", s));
                }
            }

            i += 1;

            // Stop block if direction changes after having both sides
            if !del_nums.is_empty() && !add_nums.is_empty() {
                continue; // allow combined block
            } else {
                break; // split pure blocks
            }
        }

        // Track correct match anchor
        if match_index < matches.len() {
            last_match_line = matches[match_index];
            match_index += 1;
        }

        // Determine block type
        let tag = match (del_nums.is_empty(), add_nums.is_empty()) {
            (false, true) => "d",
            (true, false) => "a",
            _ => "c",
        };

        // Correct left anchor
        let left_header = if del_nums.is_empty() {
            last_match_line
        } else {
            del_nums[0]
        };

        let right_header = if add_nums.is_empty() {
            0
        } else if add_nums.len() == 1 {
            add_nums[0]
        } else {
            add_nums[0]
        };

        let mut block = format!("{}{}{}", left_header, tag, right_header);

        // Print left lines
        for l in &del_lines {
            block.push('\n');
            block.push_str(l);
        }

        // Separator if both sides exist
        if !del_lines.is_empty() && !add_lines.is_empty() {
            block.push_str("\n---");
        }

        // Print right lines
        for r in &add_lines {
            block.push('\n');
            block.push_str(r);
        }

        blocks.push(block);
    }

    blocks
}

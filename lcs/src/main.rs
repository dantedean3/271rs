fn lcs(s1: &str, s2: &str) -> String {
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();

    let n = a.len();
    let m = b.len();

    // DP table of size (n+1) × (m+1)
    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    // Fill table
    for i in 1..=n {
        for j in 1..=m {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack
    let mut result = String::new();
    let mut i = n;
    let mut j = m;

    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.chars().rev().collect()
}

fn main() {
    let mut ss = std::env::args();
    ss.next(); // skip program name

    let s1 = ss.next().expect("missing first string");
    let s2 = ss.next().expect("missing second string");

    dbg!(lcs(&s1, &s2));
}

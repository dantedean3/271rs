use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

// ====== Config ======
const WORD_LEN: usize = 5;
const MAX_GUESSES: usize = 6;

// ANSI color codes
const RED: u8 = 31;    // not in word
const GREEN: u8 = 32;  // correct position
const YELLOW: u8 = 33; // in word, wrong position

// Box-drawing lines
const TOP: &str = "┌───┬───┬───┬───┬───┐";
const MID: &str = "├───┼───┼───┼───┼───┤";
const BOT: &str = "└───┴───┴───┴───┴───┘";

// ---------- Utilities ----------

fn clear_screen() {
    // Clear and move cursor to home
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

fn load_word_list() -> Vec<String> {
    let path = Path::new("words.txt");
    let mut words: Vec<String> = Vec::new();

    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let w = line.trim().to_lowercase();
            if w.len() == WORD_LEN && w.chars().all(|c| c.is_ascii_lowercase()) {
                words.push(w);
            }
        }
    }

    if words.is_empty() {
        // Fallback list (Latin “Sator square”)
        words = vec![
            "sator".to_string(),
            "arepo".to_string(),
            "tenet".to_string(),
            "opera".to_string(),
            "rotas".to_string(),
        ];
    }

    words
}

/// Return an unbiased random index in 0..len using bytes from /dev/random
fn rand_index(len: usize, rdr: &mut File) -> io::Result<usize> {
    // Rejection sampling to avoid modulo bias
    let bound = len as u64;
    let limit = u64::MAX - (u64::MAX % bound);

    loop {
        let mut buf = [0u8; 8];
        rdr.read_exact(&mut buf)?;
        let x = u64::from_le_bytes(buf);
        if x < limit {
            return Ok((x % bound) as usize);
        }
        // otherwise try again
    }
}

fn print_letter(ch: char, color: u8) {
    print!("│ \x1b[{}m{}\x1b[0m ", color, ch);
}

fn print_row(guess: &str, answer: &str) {
    // Simple rule: green if same position; yellow if char exists anywhere; else red.
    for (i, ch) in guess.chars().enumerate() {
        let color = if answer.chars().nth(i) == Some(ch) {
            GREEN
        } else if answer.contains(ch) {
            YELLOW
        } else {
            RED
        };
        print_letter(ch, color);
    }
    println!("│");
}

fn draw_board(guesses: &[String], answer: &str) {
    clear_screen();
    println!("{}", TOP);
    for r in 0..MAX_GUESSES {
        let row = if r < guesses.len() { &guesses[r] } else { "     " };
        print_row(row, answer);
        if r < MAX_GUESSES - 1 {
            println!("{}", MID);
        } else {
            println!("{}", BOT);
        }
    }
}

fn read_line_trimmed() -> io::Result<String> {
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_lowercase())
}

// ---------- Main ----------

fn main() -> io::Result<()> {
    let words = load_word_list();
    if words.is_empty() {
        eprintln!("No words available.");
        return Ok(());
    }

    // Choose answer using /dev/random
    let mut dev_random = File::open("/dev/random")?;
    let idx = rand_index(words.len(), &mut dev_random)?;
    let answer = &words[idx];

    // Prepare empty guesses
    let mut guesses: Vec<String> = vec!["     ".to_string(); MAX_GUESSES];

    println!("Use lowercase only. Valid guesses must be in the word list.");
    draw_board(&guesses, answer);

    for turn in 0..MAX_GUESSES {
        // Read a guess
        let guess = loop {
            print!("guess {} of {}: ", turn + 1, MAX_GUESSES);
            let _ = io::stdout().flush();
            let g = read_line_trimmed()?;

            if g.len() != WORD_LEN {
                println!("Please enter a {}-letter word.", WORD_LEN);
                continue;
            }
            if !g.chars().all(|c| c.is_ascii_lowercase()) {
                println!("Please use lowercase letters a–z only.");
                continue;
            }
            if !words.contains(&g) {
                println!("Not a valid word from the list.");
                continue;
            }
            break g;
        };

        guesses[turn] = guess.clone();
        draw_board(&guesses, answer);

        if &guess == answer {
            println!("Winner");
            return Ok(());
        }
    }

    println!("Game over :(  The word was: {}", answer);
    Ok(())
}

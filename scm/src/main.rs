use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use ed25519_dalek::{Signature, SigningKey, Verifier};
use ed25519_dalek::Signer; // required for sign()
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------
// MAIN
// ---------------------------------------------------------
fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("commit") => commit()?,
        Some("revert") => revert()?,
        Some("log") => log_cmd()?,
        Some("diff") => diff_cmd()?,
        Some("reset") => reset_cmd()?,
        Some(cmd) => {
            eprintln!("Unknown command: {cmd}");
            eprintln!("Usage: scm <commit|revert|log|diff|reset>");
        }
        None => {
            eprintln!("Usage: scm <commit|revert|log|diff|reset>");
        }
    };

    Ok(())
}

// ---------------------------------------------------------
// PATH HELPERS
// ---------------------------------------------------------

fn scm_dir() -> PathBuf {
    Path::new(".scm").to_path_buf()
}

fn log_path() -> PathBuf {
    scm_dir().join("log")
}

fn commits_dir() -> PathBuf {
    scm_dir().join("commits")
}

fn keys_dir() -> PathBuf {
    scm_dir().join("keys")
}

fn signing_key_path() -> PathBuf {
    keys_dir().join("ed25519_sk")
}

// ---------------------------------------------------------
// METADATA STRUCT
// ---------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct CommitMeta {
    root: String,
    file_hashes: BTreeMap<String, String>,
    signature: String,
}

// ---------------------------------------------------------
// HEX ENCODE / DECODE
// ---------------------------------------------------------

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(s: &str) -> io::Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid hex length"));
    }

    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    for i in (0..chars.len()).step_by(2) {
        let byte_str = format!("{}{}", chars[i], chars[i + 1]);
        out.push(u8::from_str_radix(&byte_str, 16)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid hex"))?);
    }
    Ok(out)
}

// ---------------------------------------------------------
// INIT REPO + KEY
// ---------------------------------------------------------

fn ensure_repo() -> io::Result<()> {
    if !scm_dir().exists() { fs::create_dir(scm_dir())?; }
    if !commits_dir().exists() { fs::create_dir(commits_dir())?; }
    if !log_path().exists() { OpenOptions::new().create(true).write(true).open(log_path())?; }
    if !keys_dir().exists() { fs::create_dir(keys_dir())?; }

    let _ = load_or_generate_signing_key()?;

    Ok(())
}

fn load_or_generate_signing_key() -> io::Result<SigningKey> {
    let path = signing_key_path();

    if path.exists() {
        let hex = fs::read_to_string(&path)?;
        let bytes = from_hex(hex.trim())?;
        if bytes.len() != 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Signing key must be 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        return Ok(SigningKey::from_bytes(&arr));
    }

    // Generate new 32-byte random key
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);

    let sk = SigningKey::from_bytes(&secret);

    fs::write(&path, to_hex(&secret))?;

    Ok(sk)
}

// ---------------------------------------------------------
// COMMIT ID
// ---------------------------------------------------------

fn next_commit_id(log: &Path) -> io::Result<String> {
    if !log.exists() {
        return Ok("000001".into());
    }

    let text = fs::read_to_string(log)?;
    let last = text.lines().last();

    let next_num = last
        .map(|l| l.parse::<u64>().unwrap_or(0) + 1)
        .unwrap_or(1);

    Ok(format!("{:06}", next_num))
}

// ---------------------------------------------------------
// TRACKED FILES
// ---------------------------------------------------------

fn tracked_files() -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy();
            if name != ".scm" {
                files.push(path);
            }
        }
    }
    Ok(files)
}

// ---------------------------------------------------------
// COPY + RESTORE FILE SNAPSHOTS
// ---------------------------------------------------------

fn copy_files(files: &[PathBuf], commit_dir: &Path) -> io::Result<()> {
    for f in files {
        let name = f.file_name().unwrap();
        fs::copy(f, commit_dir.join(name))?;
    }
    Ok(())
}

fn restore_from(commit_dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(commit_dir)? {
        let entry = entry?;
        let name = entry.file_name();

        if name == "meta.json" {
            continue;
        }

        fs::copy(entry.path(), Path::new(".").join(name))?;
    }
    Ok(())
}

// ---------------------------------------------------------
// APPEND LOG FILE
// ---------------------------------------------------------

fn append_log(id: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())?;

    writeln!(file, "{id}")?;
    Ok(())
}

// ---------------------------------------------------------
// HASHING + MERKLE TREE
// ---------------------------------------------------------

fn compute_hashes(dir: &Path) -> io::Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if name == "meta.json" { continue; }

            let data = fs::read(&path)?;
            let hash = Sha256::digest(&data);
            out.insert(name, to_hex(&hash));
        }
    }

    Ok(out)
}

fn merkle_root(hashes: &[String]) -> String {
    if hashes.len() == 1 {
        return hashes[0].clone();
    }

    let mut layer = hashes.to_vec();

    while layer.len() > 1 {
        let mut next = Vec::new();
        let mut i = 0;

        while i < layer.len() {
            let left = from_hex(&layer[i]).unwrap();
            let right = if i + 1 < layer.len() {
                from_hex(&layer[i + 1]).unwrap()
            } else {
                left.clone()
            };

            let mut hasher = Sha256::new();
            hasher.update(left);
            hasher.update(right);

            next.push(to_hex(&hasher.finalize()));

            i += 2;
        }

        layer = next;
    }

    layer[0].clone()
}

fn compute_meta(commit_dir: &Path) -> io::Result<CommitMeta> {
    let file_hashes = compute_hashes(commit_dir)?;
    let values: Vec<String> = file_hashes.values().cloned().collect();

    let root = merkle_root(&values);

    // sign the root
    let sk = load_or_generate_signing_key()?;
    let root_bytes = from_hex(&root)?;
    let sig = sk.sign(&root_bytes);

    Ok(CommitMeta {
        root,
        file_hashes,
        signature: to_hex(&sig.to_bytes()),
    })
}

fn verify_meta(commit_dir: &Path, meta: &CommitMeta) -> io::Result<bool> {
    // Recompute file hashes
    let recomputed = compute_hashes(commit_dir)?;

    if recomputed != meta.file_hashes {
        eprintln!("Hash mismatch detected.");
        return Ok(false);
    }

    // Recompute Merkle root
    let values: Vec<String> = recomputed.values().cloned().collect();
    let new_root = merkle_root(&values);

    if new_root != meta.root {
        eprintln!("Merkle root mismatch.");
        return Ok(false);
    }

    // Verify signature
    let sk = load_or_generate_signing_key()?;
    let vk = sk.verifying_key();

    let root_bytes = from_hex(&meta.root)?;
    let sig_bytes = from_hex(&meta.signature)?;
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);

    let sig = Signature::from_bytes(&sig_arr);

    if vk.verify(&root_bytes, &sig).is_err() {
        eprintln!("Signature verification failed.");
        return Ok(false);
    }

    Ok(true)
}

// ---------------------------------------------------------
// COMMAND: COMMIT
// ---------------------------------------------------------

fn commit() -> io::Result<()> {
    ensure_repo()?;

    let id = next_commit_id(&log_path())?;
    let commit_dir = commits_dir().join(&id);
    fs::create_dir_all(&commit_dir)?;

    let files = tracked_files()?;
    copy_files(&files, &commit_dir)?;

    let meta = compute_meta(&commit_dir)?;
    fs::write(
        commit_dir.join("meta.json"),
        serde_json::to_string_pretty(&meta).unwrap(),
    )?;

    append_log(&id)?;

    println!("Committed as {}", id);
    Ok(())
}

// ---------------------------------------------------------
// COMMAND: REVERT
// ---------------------------------------------------------

fn revert() -> io::Result<()> {
    let contents = fs::read_to_string(log_path())?;
    let commits: Vec<String> = contents.lines().map(|s| s.to_string()).collect();

    if commits.len() < 2 {
        eprintln!("Not enough commits to revert.");
        return Ok(());
    }

    let target = commits[commits.len() - 2].clone(); // FIXED
    let commit_dir = commits_dir().join(&target);

    let meta_path = commit_dir.join("meta.json");
    if !meta_path.exists() {
        eprintln!("Missing metadata. Cannot verify commit.");
        return Ok(());
    }

    let meta: CommitMeta =
        serde_json::from_str(&fs::read_to_string(meta_path)?).unwrap();

    if !verify_meta(&commit_dir, &meta)? {
        eprintln!("Integrity or signature verification failed.");
        return Ok(());
    }

    restore_from(&commit_dir)?;

    // Remove latest commit
    let mut new_log = commits;
    new_log.pop();

    fs::write(log_path(), new_log.join("\n") + "\n")?;

    println!("Reverted to commit {}", target);
    Ok(())
}

// ---------------------------------------------------------
// COMMAND: LOG
// ---------------------------------------------------------

fn log_cmd() -> io::Result<()> {
    let contents = fs::read_to_string(log_path())?;
    let list: Vec<_> = contents.lines().collect();

    if list.is_empty() {
        println!("No commits yet.");
        return Ok(());
    }

    println!("Commit history:");
    for c in list.iter().rev() {
        println!("  {}", c);
    }

    Ok(())
}

// ---------------------------------------------------------
// COMMAND: DIFF
// ---------------------------------------------------------

fn diff_cmd() -> io::Result<()> {
    let contents = fs::read_to_string(log_path())?;
    let commits: Vec<_> = contents.lines().collect();

    if commits.is_empty() {
        eprintln!("No commits found.");
        return Ok(());
    }

    let last = commits.last().unwrap();
    let commit_dir = commits_dir().join(last);

    println!("Diff vs commit {}:", last);

    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();

        if path.is_file() {
            let committed = commit_dir.join(&name);
            if committed.exists() {
                let wc = fs::read_to_string(&path)?;
                let old = fs::read_to_string(committed)?;
                if wc != old {
                    println!("* {} modified", name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------
// COMMAND: RESET
// ---------------------------------------------------------

fn reset_cmd() -> io::Result<()> {
    let contents = fs::read_to_string(log_path())?;
    let mut commits: Vec<String> = contents.lines().map(|l| l.to_string()).collect();

    if commits.is_empty() {
        eprintln!("No commits to remove.");
        return Ok(());
    }

    let last = commits.pop().unwrap();
    let dir = commits_dir().join(&last);
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }

    fs::write(log_path(), commits.join("\n") + "\n")?;

    println!("Removed commit {}", last);
    Ok(())
}

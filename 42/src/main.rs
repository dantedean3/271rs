use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[inline(always)]
fn rotr(x: u64, n: u32) -> u64 { (x >> n) | (x << (64 - n)) }
#[inline(always)]
fn ch(x: u64, y: u64, z: u64) -> u64 { (x & y) ^ (!x & z) }
#[inline(always)]
fn maj(x: u64, y: u64, z: u64) -> u64 { (x & y) ^ (x & z) ^ (y & z) }
#[inline(always)]
fn big_sigma0(x: u64) -> u64 { rotr(x, 28) ^ rotr(x, 34) ^ rotr(x, 39) }
#[inline(always)]
fn big_sigma1(x: u64) -> u64 { rotr(x, 14) ^ rotr(x, 18) ^ rotr(x, 41) }
#[inline(always)]
fn small_sigma0(x: u64) -> u64 { rotr(x, 1) ^ rotr(x, 8) ^ (x >> 7) }
#[inline(always)]
fn small_sigma1(x: u64) -> u64 { rotr(x, 19) ^ rotr(x, 61) ^ (x >> 6) }

const K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

struct Sha512 {
    h: [u64; 8],
    buffer: [u8; 128],
    buf_len: usize,
    total_len_bytes: u128,
}

impl Sha512 {
    fn new() -> Self {
        Self {
            h: [
                0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b,
                0xa54ff53a5f1d36f1, 0x510e527fade682d1, 0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
            ],
            buffer: [0u8; 128],
            buf_len: 0,
            total_len_bytes: 0,
        }
    }

    fn process_block(&mut self, block: &[u8; 128]) {
        let mut w = [0u64; 80];
        for t in 0..16 {
            let i = t * 8;
            w[t] = u64::from_be_bytes(block[i..i + 8].try_into().unwrap());
        }
        for t in 16..80 {
            w[t] = small_sigma1(w[t - 2])
                .wrapping_add(w[t - 7])
                .wrapping_add(small_sigma0(w[t - 15]))
                .wrapping_add(w[t - 16]);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) =
            (self.h[0], self.h[1], self.h[2], self.h[3],
             self.h[4], self.h[5], self.h[6], self.h[7]);

        for t in 0..80 {
            let t1 = h
                .wrapping_add(big_sigma1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let t2 = big_sigma0(a).wrapping_add(maj(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
        self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g);
        self.h[7] = self.h[7].wrapping_add(h);
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total_len_bytes = self.total_len_bytes.wrapping_add(data.len() as u128);

        if self.buf_len > 0 {
            let need = 128 - self.buf_len;
            let take = need.min(data.len());
            self.buffer[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];

            if self.buf_len == 128 {
                let block = self.buffer; // ✅ FIX for borrow checker
                self.process_block(&block);
                self.buf_len = 0;
            }
        }

        while data.len() >= 128 {
            self.process_block(data[..128].try_into().unwrap());
            data = &data[128..];
        }

        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    fn finalize(mut self) -> [u8; 64] {
        let mut pad = [0u8; 256];
        pad[0] = 0x80;
        let bit_len = self.total_len_bytes.wrapping_mul(8);
        let len_mod = (self.buf_len + 1) % 128;
        let pad_zeroes = if len_mod <= 112 { 112 - len_mod } else { 240 - len_mod };
        let len_bytes = bit_len.to_be_bytes();
        self.update(&pad[..1 + pad_zeroes]);
        self.update(&len_bytes);
        let mut out = [0u8; 64];
        for (i, word) in self.h.iter().enumerate() {
            out[i * 8..i * 8 + 8].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

fn sha512_reader<R: Read>(mut r: R) -> io::Result<[u8; 64]> {
    let mut h = Sha512::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 { break; }
        h.update(&buf[..n]);
    }
    Ok(h.finalize())
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn print_one_result<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let p = path.as_ref();
    let file = File::open(p)?;
    let digest = sha512_reader(file)?;
    println!("{}  {}", to_hex(&digest), p.display());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        let digest = sha512_reader(io::stdin()).unwrap();
        println!("{}  -", to_hex(&digest));
        return;
    }
    let mut exit_code = 0;
    for a in &args {
        if let Err(e) = print_one_result(a) {
            eprintln!("sha512: {a}: {e}");
            exit_code = 1;
        }
    }
    std::process::exit(exit_code);
}

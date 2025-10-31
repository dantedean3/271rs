use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero, One, Euclid};
use sha2::{Digest, Sha512};

// alias for convenience
pub type Point = Vec<BigInt>; // [x, y]

// --- Global Helpers (no curve constants needed here) ---

// H(m: bytes) -> bytes (SHA-512)
fn h(m: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(m);
    hasher.finalize().to_vec()
}

// bit(h: bytes, i: int) -> int (currently unused, silence warning)
#[allow(dead_code)]
fn bit(h_val: &[u8], i: usize) -> u8 {
    let byte_i = i / 8;
    let bit_i = i % 8;
    if byte_i >= h_val.len() {
        0
    } else {
        (h_val[byte_i] >> bit_i) & 1
    }
}

// expmod(b:int,e:int,m:int) -> int (modular exponentiation)
pub fn expmod(b_val: &BigInt, e: &BigInt, m: &BigInt) -> BigInt {
    let mut base = b_val.mod_floor(m);
    let mut exp = e.clone();
    let mut acc = BigInt::one();
    while exp > BigInt::zero() {
        if (&exp & BigInt::one()).is_one() {
            acc = (acc * &base).rem_euclid(m);
        }
        base = (&base * &base).rem_euclid(m);
        exp >>= 1;
    }
    acc
}

// inv(x:int, q: &BigInt) -> int (multiplicative inverse mod q)
pub fn inv(x: &BigInt, q: &BigInt) -> BigInt {
    let e = q - BigInt::from(2);
    expmod(x, &e, q)
}

// xrecover(y): recover x from y
pub fn xrecover(y: &BigInt, q: &BigInt, d: &BigInt, i_const: &BigInt) -> BigInt {
    let one = BigInt::one();
    let y2 = (y * y).rem_euclid(q);
    let num = (y2.clone() - &one).rem_euclid(q);
    let den = (d * &y2 + &one).rem_euclid(q);
    let den_inv = inv(&den, q);
    let xx = (num * den_inv).rem_euclid(q);

    // exponent (q+3)/8
    let exp = (q + BigInt::from(3)) >> 3;
    let mut x = expmod(&xx, &exp, q);

    // If x^2 != xx, multiply by sqrt(-1)
    if (&x * &x - &xx).rem_euclid(q) != BigInt::zero() {
        x = (x * i_const).rem_euclid(q);
    }

    // Ensure even representative
    if x.is_odd() {
        x = (q - x).rem_euclid(q);
    }

    x
}

// --- Core Functions ---

// Edwards group law (twisted Edwards)
fn edwards(p: &Point, q_val: &Point, q: &BigInt, d: &BigInt) -> Point {
    let (x1, y1) = (&p[0], &p[1]);
    let (x2, y2) = (&q_val[0], &q_val[1]);
    let one = BigInt::one();

    let x1y2 = (x1 * y2).rem_euclid(q);
    let y1x2 = (y1 * x2).rem_euclid(q);
    let y1y2 = (y1 * y2).rem_euclid(q);
    let x1x2 = (x1 * x2).rem_euclid(q);
    let dxxyy = (d * &x1x2 * &y1y2).rem_euclid(q);

    let num_x = (x1y2 + y1x2).rem_euclid(q);
    let den_x = (one.clone() + dxxyy.clone()).rem_euclid(q);
    let num_y = (y1y2 + x1x2).rem_euclid(q);
    let den_y = (one - dxxyy).rem_euclid(q);

    let x = (num_x * inv(&den_x, q)).rem_euclid(q);
    let y = (num_y * inv(&den_y, q)).rem_euclid(q);

    vec![x, y]
}

// Scalar multiplication
fn scalarmult(p: &Point, e: &BigInt, q: &BigInt, d: &BigInt) -> Point {
    let mut acc: Point = vec![BigInt::zero(), BigInt::one()];
    let mut base = p.clone();
    let mut k = e.clone();

    while k > BigInt::zero() {
        if (&k & BigInt::one()).is_one() {
            acc = edwards(&acc, &base, q, d);
        }
        base = edwards(&base, &base, q, d);
        k >>= 1;
    }
    acc
}

// Encode integer to little-endian
fn encodeint(y: &BigInt, b: usize) -> Vec<u8> {
    let nbytes = b / 8;
    let mut x = y.clone();
    let mut out = vec![0u8; nbytes];
    for i in 0..nbytes {
        out[i] = (&x & BigInt::from(255)).to_u8().unwrap();
        x >>= 8;
    }
    out
}

// Encode point (with x parity bit)
fn encodepoint(p: &Point, b: usize) -> Vec<u8> {
    let nbytes = b / 8;
    let x = &p[0];
    let y = &p[1];
    let mut enc = encodeint(y, b);
    let sign = (x & BigInt::one()).to_u8().unwrap();
    enc[nbytes - 1] |= sign << 7;
    enc
}

// Public key generation
pub fn publickey(sk: &[u8], b: usize, q: &BigInt, d: &BigInt, b_point: &Point) -> Vec<u8> {
    let digest = h(sk);

    let mut a_bytes = digest[0..32].to_vec();
    a_bytes[0] &= 248;
    a_bytes[31] &= 63;
    a_bytes[31] |= 64;

    let mut a = BigInt::zero();
    for (i, &byte) in a_bytes.iter().enumerate() {
        a += BigInt::from(byte) << (8 * i);
    }

    let a_point = scalarmult(b_point, &a, q, d);
    encodepoint(&a_point, b)
}

// Hint helper
fn hint(m: &[u8], _b: usize) -> BigInt {
    let hh = h(m);
    let mut x = BigInt::zero();
    for (i, &byte) in hh.iter().enumerate() {
        x += BigInt::from(byte) << (8 * i);
    }
    x
}

// Signature generation
pub fn signature(
    m: &[u8],
    sk: &[u8],
    pk: &[u8],
    b: usize,
    q: &BigInt,
    l: &BigInt,
    d: &BigInt,
    b_point: &Point,
) -> Vec<u8> {
    let digest = h(sk);

    let mut a_bytes = digest[0..32].to_vec();
    a_bytes[0] &= 248;
    a_bytes[31] &= 63;
    a_bytes[31] |= 64;
    let mut a = BigInt::zero();
    for (i, &byte) in a_bytes.iter().enumerate() {
        a += BigInt::from(byte) << (8 * i);
    }

    let prefix = &digest[32..64];

    let mut r_inp = Vec::with_capacity(prefix.len() + m.len());
    r_inp.extend_from_slice(prefix);
    r_inp.extend_from_slice(m);
    let r = hint(&r_inp, b).rem_euclid(l);

    let r_point = scalarmult(b_point, &r, q, d);
    let r_enc = encodepoint(&r_point, b);

    let mut h_in = Vec::with_capacity(r_enc.len() + pk.len() + m.len());
    h_in.extend_from_slice(&r_enc);
    h_in.extend_from_slice(pk);
    h_in.extend_from_slice(m);
    let h_val = hint(&h_in, b).rem_euclid(l);

    let s = (r + h_val * a).rem_euclid(l);
    let s_enc = encodeint(&s, b);

    let mut sig = Vec::with_capacity(64);
    sig.extend_from_slice(&r_enc);
    sig.extend_from_slice(&s_enc);
    sig
}

// On-curve check (fixed version)
fn isoncurve(p: &Point, q: &BigInt, d: &BigInt) -> bool {
    if p.len() != 2 {
        return false;
    }
    let (x, y) = (&p[0], &p[1]);
    let x2 = (x * x).rem_euclid(q);
    let y2 = (y * y).rem_euclid(q);
    let left = (y2.clone() - &x2).rem_euclid(q);
    let right = (BigInt::one() + d * &x2 * &y2).rem_euclid(q);
    (left - right).rem_euclid(q) == BigInt::zero()
}

// Decode integer from bytes
fn decodeint(s: &[u8], _b: usize) -> BigInt {
    let mut x = BigInt::zero();
    for (i, &byte) in s.iter().enumerate() {
        x += BigInt::from(byte) << (8 * i);
    }
    x
}

// Decode point from bytes
fn decodepoint(
    s: &[u8],
    b: usize,
    q: &BigInt,
    d: &BigInt,
    i_const: &BigInt,
) -> Result<Point, &'static str> {
    let nbytes = b / 8;
    if s.len() != nbytes {
        return Err("bad length");
    }

    let mut y_bytes = s.to_vec();
    let sign = (y_bytes[nbytes - 1] >> 7) & 1;
    y_bytes[nbytes - 1] &= 0x7F;

    let y = decodeint(&y_bytes, b).rem_euclid(q);
    if y >= *q {
        return Err("y out of range");
    }

    let mut x = xrecover(&y, q, d, i_const);
    if (x.clone() & BigInt::one()).to_u8().unwrap() != sign {
        x = (q - x).rem_euclid(q);
    }

    let p = vec![x, y];
    if !isoncurve(&p, q, d) {
        return Err("point not on curve");
    }
    Ok(p)
}

// Signature verification
pub fn checkvalid(
    s: &[u8],
    m: &[u8],
    pk: &[u8],
    b: usize,
    q: &BigInt,
    d: &BigInt,
    i_const: &BigInt,
    b_point: &Point,
) -> bool {
    let nbytes = b / 8;
    if s.len() != 2 * nbytes || pk.len() != nbytes {
        return false;
    }

    let r_enc = &s[0..nbytes];
    let s_enc = &s[nbytes..2 * nbytes];

    let r_point = match decodepoint(r_enc, b, q, d, i_const) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let s_int = decodeint(s_enc, b);
    let a_point = match decodepoint(pk, b, q, d, i_const) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let l_suffix =
        BigInt::parse_bytes(b"27742317777372353535851937790883648493", 10).unwrap();
    let l = (BigInt::from(2).pow(252)) + l_suffix;

    let mut h_in = Vec::with_capacity(nbytes * 2 + m.len());
    h_in.extend_from_slice(r_enc);
    h_in.extend_from_slice(pk);
    h_in.extend_from_slice(m);
    let h_val = hint(&h_in, b).rem_euclid(&l);

    let sb = scalarmult(b_point, &s_int.rem_euclid(&l), q, d);
    let ha = scalarmult(&a_point, &h_val, q, d);
    let r_plus_ha = edwards(&r_point, &ha, q, d);

    encodepoint(&sb, b) == encodepoint(&r_plus_ha, b)
}

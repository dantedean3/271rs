use num::*;
use num_bigint::BigInt;
use num_traits::Euclid;

fn main() {
    // --- 1. Constant Setup  ---
    const B: usize = 256;
    
    // q: int = 2**255 - 19
    let q = BigInt::from(2).pow(255) - BigInt::from(19);

    // l: int = 2**252 + 27742317777372353535851937790883648493
    let l_suffix = BigInt::parse_bytes(b"27742317777372353535851937790883648493", 10).unwrap();
    let l = BigInt::from(2).pow(252) + l_suffix;

    // d: int = -121665 * inv(121666)
    let n_d = BigInt::from(-121665);
    let d_inv = inv(&BigInt::from(121666), &q);
    let d = (n_d * d_inv).rem_euclid(&q);

    // I: int = expmod(2,(q-1)//4,q)
    let exponent_i = (&q - BigInt::from(1)) / BigInt::from(4);
    let i_const = expmod(&BigInt::from(2), &exponent_i, &q);
    
    // By : int = 4 * inv(5)
    let by = (BigInt::from(4) * inv(&BigInt::from(5), &q)).rem_euclid(&q);
    
    // Bx : int = xrecover(By)
    let bx = xrecover(&by, &q, &d, &i_const);
    
    // B : List[int] = [Bx % q, By % q]
    let b_point: Vec<BigInt> = vec![bx.rem_euclid(&q), by.rem_euclid(&q)];

    // println!("Ed25519 Constant Initialization Complete.");
    
    // --- 2. Test Case ---
    // Line 2 of this file. https://ed25519.cr.yp.to/python/sign.input
    
    // Secret Key (32-bytes, all zeros)
    // 4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c
    let sk = vec![0x4c, 0xcd, 0x08, 0x9b, 0x28, 0xff, 0x96, 0xda, 0x9d, 0xb6, 0xc3, 0x46, 0xec, 0x11, 0x4e, 0x0f, 0x5b, 0x8a, 0x31, 0x9f, 0x35, 0xab, 0xa6, 0x24, 0xda, 0x8c, 0xf6, 0xed, 0x4f, 0xb8, 0xa6, 0xfb, 0x3d, 0x40, 0x17, 0xc3, 0xe8, 0x43, 0x89, 0x5a, 0x92, 0xb7, 0x0a, 0xa7, 0x4d, 0x1b, 0x7e, 0xbc, 0x9c, 0x98, 0x2c, 0xcf, 0x2e, 0xc4, 0x96, 0x8c, 0xc0, 0xcd, 0x55, 0xf1, 0x2a, 0xf4, 0x66, 0x0c];

    // Message
    // 72
    let m = b"r";

    
    // 3. Generate Public Key
    let pk = publickey(&sk, B, &q, &d, &b_point);

    // 4. Create Signature
    let sig = signature(m, &sk, &pk, B, &q, &l, &d, &b_point);
    
    // 5. Verify Signature
    dbg!(checkvalid(&sig, m, &pk, B, &q, &d, &i_const, &b_point));
}
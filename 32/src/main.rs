use std::mem::size_of_val;

fn main() {
    let a: [u64; 4] = [
        0x1111111111110000,
        0x1111000011001100,
        0x1100110010101010,
        0x0123_4567_89AB_CDEF,
    ];

    println!("*Rotates use a decimal shift value, but print in hexadecimal:\n");

    println!(
        "choice(\n{:016X},\n{:016X},\n{:016X}) = \n--------\n{:016X}\n\n",
        a[0],
        a[1],
        a[2],
        macros::choice!(a[0], a[1], a[2])
    );

    println!(
        "median(\n{:016X},\n{:016X},\n{:016X}) = \n--------\n{:016X}\n\n",
        a[0],
        a[1],
        a[2],
        macros::median!(a[0], a[1], a[2])
    );

    println!("*Rotates use a decimal shift value, but print in hexadecimal:\n");

    println!(
        "rotate!(\n{:016X}, 04) = \n--------\n{:016X}\n\n",
        a[3],
        macros::rotate!(a[3], 4)
    );
    println!(
        "rotate!(\n{:016X}, 08) = \n--------\n{:016X}\n\n",
        a[3],
        macros::rotate!(a[3], 8)
    );
    println!(
        "rotate!(\n{:016X}, 12) = \n--------\n{:016X}\n\n",
        a[3],
        macros::rotate!(a[3], 12)
    );
    println!(
        "rotate!(\n{:016X}, 02) = \n--------\n{:016X}\n\n",
        0x1000_u64,
        macros::rotate!(0x1000_u64, 2)
    );
    println!(
        "rotate!(\n{:016X}, 30) = \n--------\n{:016X}\n\n",
        0x1000_u64,
        macros::rotate!(0x1000_u64, 30)
    );

    println!(
        "Bit width of a[3] is {} bits",
        size_of_val(&a[3]) * 8
    );
}

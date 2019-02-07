use {
    orcrist::{FromFixedBytes, Le},
    std::io::Cursor,
};

#[derive(Debug, Eq, FromFixedBytes, PartialEq)]
struct Wat(Le<u32>);

#[derive(Debug, Eq, FromFixedBytes, PartialEq)]
struct Asdf {
    meh: u8,
    blarg: Le<u16>,
}

fn main() {
    assert_eq!(
        Wat(Le(1)),
        Wat::from_fixed_bytes(&mut Cursor::new(b"\x01\x00\x00\x00")).unwrap()
    );
    assert_eq!(
        Wat(16u32.into()),
        Wat::from_fixed_bytes(&mut Cursor::new(b"\x10\x00\x00\x00")).unwrap()
    );

    assert_eq!(
        Asdf {
            meh: 5.into(),
            blarg: 27.into(),
        },
        Asdf::from_fixed_bytes(&mut Cursor::new(b"\x05\x1B\x00")).unwrap()
    );

    println!(
        "Derive works! See the source of this example ({}) for more details.",
        file!()
    );
    println!("Failures would look something like this:");
    println!(
        "{}",
        Wat::from_fixed_bytes(&mut Cursor::new(b"")).unwrap_err()
    );
    println!(
        "{}",
        Asdf::from_fixed_bytes(&mut Cursor::new(b"")).unwrap_err()
    );
}

use {
    orcrist::{ByteReadFailure, FromFixedBytes, Le, PrimitiveField},
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        io::{Cursor, Read},
    },
};

#[derive(Debug, Eq, PartialEq)]
struct Wat(Le<u32>);

#[derive(Debug)]
struct WatField(PrimitiveField);

impl Display for WatField {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl FromFixedBytes for Wat {
    type FieldEnum = WatField;

    fn from_fixed_bytes<R: Read>(stream: &mut R) -> Result<Self, ByteReadFailure<Self::FieldEnum>> {
        Ok(Self(FromFixedBytes::from_fixed_bytes(stream).map_err(|e| e.map_field("Wat",  WatField))?))
    }
}

fn main() {
    assert_eq!(Wat(Le(1)), Wat::from_fixed_bytes(&mut Cursor::new(b"\x01\x00\x00\x00")).unwrap());
    println!("Implementing `FromFixedBytes` manually using library newtypes works! See the source of this example ({}) for more details.", file!());
    println!(
        "A failure would look something like this: {}",
        Wat::from_fixed_bytes(&mut Cursor::new(&[0u8])).unwrap_err()
    );
}

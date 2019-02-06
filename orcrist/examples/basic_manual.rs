use {
    byteorder::{LittleEndian, ReadBytesExt},
    orcrist::{ByteReadFailure, FromFixedBytes},
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        io::{Cursor, Read},
    },
};

#[derive(Debug, Eq, PartialEq)]
struct Wat(u32);

#[derive(Debug)]
enum WatFields {
    InnerValue,
}

impl Display for WatFields {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::WatFields::*;

        match self {
            InnerValue => write!(f, "inner value")?,
        }

        Ok(())
    }
}

impl crate::FromFixedBytes for Wat {
    type FieldEnum = WatFields;

    fn from_fixed_bytes<R: Read>(stream: &mut R) -> Result<Self, ByteReadFailure<Self::FieldEnum>> {
        use self::WatFields::*;

        Ok(Self(stream.read_u32::<LittleEndian>().map_err(|e| {
            ByteReadFailure {
                field: InnerValue,
                type_name: "Wat",
                inner: e,
            }
        })?))
    }
}

fn main() {
    assert_eq!(
        Wat(1),
        Wat::from_fixed_bytes(&mut Cursor::new(b"\x01\x00\x00\x00")).unwrap()
    );
    println!("Implementing `FromFixedBytes` manually works! See the source of this example ({}) for more details.", file!());
    println!(
        "A failure would look something like this: {}",
        Wat::from_fixed_bytes(&mut Cursor::new(&[0u8])).unwrap_err()
    );
}

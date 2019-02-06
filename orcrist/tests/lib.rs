mod basic_manual {
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

    impl FromFixedBytes for Wat {
        type FieldEnum = WatFields;

        fn from_fixed_bytes<R: Read>(
            stream: &mut R,
        ) -> Result<Self, ByteReadFailure<Self::FieldEnum>> {
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

    #[test]
    fn test() {
        assert_eq!(
            Wat(1),
            Wat::from_fixed_bytes(&mut Cursor::new(b"\x01\x00\x00\x00")).unwrap()
        );
    }
}

#[cfg(feature = "derive")]
mod basic_derive {
    use {
        orcrist::{FromFixedBytes, Le},
        std::io::Cursor,
    };

    #[derive(Debug, Eq, FromFixedBytes, PartialEq)]
    struct Wat(Le<u32>);

    #[test]
    fn test() {
        assert_eq!(
            Wat(Le(1)),
            Wat::from_fixed_bytes(&mut Cursor::new(b"\x01\x00\x00\x00")).unwrap()
        );
    }
}

use {
    named_type::NamedType,
    named_type_derive::NamedType,
    std::{
        error::Error as ErrorTrait,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        io::{Error as IoError, Read},
        mem::{size_of, transmute, uninitialized},
    },
};

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate orcrist_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use orcrist_derive::*;

pub struct ByteReadFailure<FieldEnum> {
    pub inner: IoError,
    pub type_name: &'static str,
    pub field: FieldEnum,
}

impl<FieldEnum> ByteReadFailure<FieldEnum> {
    pub fn map_field<OtherFieldEnum, F>(
        self,
        other_type_name: &'static str,
        f: F,
    ) -> ByteReadFailure<OtherFieldEnum>
    where
        F: FnOnce(FieldEnum) -> OtherFieldEnum,
    {
        let Self {
            inner,
            type_name: _type_name,
            field,
        } = self;

        ByteReadFailure {
            inner,
            type_name: other_type_name,
            field: f(field),
        }
    }
}

impl<FieldEnum: Debug> Debug for ByteReadFailure<FieldEnum> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let ByteReadFailure {
            inner,
            type_name,
            field,
        } = self;
        f.debug_struct("ByteReadFailure")
            .field("inner", inner)
            .field("type_name", type_name)
            .field("field", field)
            .finish()
    }
}

impl<FieldEnum: Display> Display for ByteReadFailure<FieldEnum> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let ByteReadFailure {
            inner,
            type_name,
            field,
        } = self;
        write!(f, "unable to read {} of {}: {}", field, type_name, inner)
    }
}

impl<FieldEnum: Debug + Display> ErrorTrait for ByteReadFailure<FieldEnum> {}

pub trait FromFixedBytes: Sized {
    type FieldEnum: Sized + Debug + Display;

    fn from_fixed_bytes<R: Read>(stream: &mut R) -> Result<Self, ByteReadFailure<Self::FieldEnum>>;
}

macro_rules! implement_fmt_for_newtype {
    ($name: ident, $fmt_trait: ident) => {
        impl<T: std::fmt::$fmt_trait> std::fmt::$fmt_trait for $name<T> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                self.0.fmt(f)
            }
        }
    };
}

macro_rules! declare_endian_wrapper {
    ($name: ident) => {
        #[derive(Clone, Copy, Default, Hash, Eq, NamedType, Ord, PartialEq, PartialOrd)]
        pub struct $name<T>(pub T);

        implement_fmt_for_newtype!($name, Display);
        implement_fmt_for_newtype!($name, Debug);
        implement_fmt_for_newtype!($name, Octal);
        implement_fmt_for_newtype!($name, Binary);
        implement_fmt_for_newtype!($name, LowerHex);
        implement_fmt_for_newtype!($name, UpperHex);

        impl<T> From<T> for $name<T> {
            fn from(t: T) -> Self {
                $name(t)
            }
        }
    };
}
declare_endian_wrapper!(Le);
declare_endian_wrapper!(Be);
declare_endian_wrapper!(Ne);

#[derive(Debug)]
pub struct PrimitiveField;

impl Display for PrimitiveField {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "primitive value")
    }
}

pub struct ArrayElement<T> {
    index: usize,
    field: T,
}

impl<T: Debug> Debug for ArrayElement<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self {
            index,
            field,
        } = self;

        f.debug_struct("ArrayElement")
            .field("index", index)
            .field("field", field)
            .finish()
    }
}

impl<T> Display for ArrayElement<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self {
            index,
            field: _field,
        } = self;

        write!(f, "element at index {}", index)
    }
}

// impl<A: Array<Item=$primitive_name>> FromFixedBytes for $newtype_name<A> {
//     type FieldEnum = ArrayElement;

//     fn from_fixed_bytes<R: Read>(stream: &mut R) -> Result<Self, ByteReadFailure<Self::FieldEnum>> {
//         let raw = unsafe {
//             uninitialized::<[u8; size_of::<Array>()]>()
//         };
//         for (index, raw_element) in raw.iter_mut().enumerate() {
//             *raw_element = <$primitive_name as FromFixedBytes>::from_fixed_bytes(stream).map_err(|e| e.map_field(
//                 stringify!(array of $primitive_name),
//                 |field| ArrayElement {
//                     index,
//                     field,
//                 },
//             ))?
//         }
//         let initialized = unsafe {
//             transmute(raw)
//         };
//         Ok($newtype_name(initialized))
//     }
// }

macro_rules! impl_primitive_conversion {
    ($newtype_name: ident, $primitive_name: ident, $array_conv_fn: ident) => {
        impl FromFixedBytes for $newtype_name<$primitive_name> {
            type FieldEnum = PrimitiveField;

            fn from_fixed_bytes<R: Read>(
                stream: &mut R,
            ) -> Result<Self, ByteReadFailure<Self::FieldEnum>> {
                let mut buf = unsafe {
                    // This is safe because we will be initializing the entire array without reading first.
                    uninitialized::<[u8; size_of::<$primitive_name>()]>()
                };
                stream.read_exact(&mut buf).map_err(|e| ByteReadFailure {
                    field: PrimitiveField,
                    type_name: $newtype_name::<$primitive_name>::type_name(),
                    inner: e,
                })?;
                Ok($newtype_name($primitive_name::$array_conv_fn(buf)))
            }
        }
    };
}

macro_rules! impl_primitive_conversions {
    ($newtype_name: ident, $array_conv_fn: ident) => {
        impl_primitive_conversion!($newtype_name, i8, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, i16, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, i32, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, i64, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, i128, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, u8, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, u16, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, u32, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, u64, $array_conv_fn);
        impl_primitive_conversion!($newtype_name, u128, $array_conv_fn);
    };
}

impl_primitive_conversions!(Le, from_le_bytes);
impl_primitive_conversions!(Be, from_be_bytes);
impl_primitive_conversions!(Ne, from_ne_bytes);

macro_rules! impl_array_conversion {
    ($size: expr) => {
        impl<InnerFieldEnum: Debug + Display, T: Debug + FromFixedBytes<FieldEnum=InnerFieldEnum> + Sized> FromFixedBytes for [T; $size] {
            type FieldEnum = ArrayElement<InnerFieldEnum>;

            fn from_fixed_bytes<R: Read>(stream: &mut R) -> Result<Self, ByteReadFailure<Self::FieldEnum>> {
                let raw = unsafe {
                    uninitialized::<[u8; $size * size_of::<T>()]>()
                };
                for (index, raw_element) in raw.iter_mut().enumerate() {
                    *raw_element = <T as FromFixedBytes>::from_fixed_bytes(stream).map_err(|e| e.map_field(
                        "array",
                        |field| ArrayElement {
                            index,
                            field,
                        },
                    ))?
                }
                Ok(unsafe {
                    transmute(raw)
                })
            }
        }
    };
}

impl_array_conversion!(0);
impl_array_conversion!(1);

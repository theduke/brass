#[macro_export]
macro_rules! make_str_enum {
    (
        $enum_name:ident {
            $( $name:ident = $num:literal = $value:literal, )*
            ;
            $last_name:ident = $last_num:literal = $last_value:literal
        }
    ) => {
        #[repr(u16)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $enum_name {
            $( $name = $num, )*
            $last_name = $last_num,
        }

        impl $enum_name {
            /// The number of [`Attr`] variants.
            ///
            /// Useful for things like lookup arrays.
            pub(crate) const fn variant_count() -> usize {
                $last_num + 1
            }

            /// Convert to numeric representation.
            pub(crate) const fn as_u16(self) -> u16 {
                self as u16
            }

            pub(crate) const fn from_u16(value: u16) -> Option<Self> {
                match value {
                    $(
                        $num => Some(Self::$name),
                    )*
                    $last_num => Some(Self::$last_name),
                    _ => None,
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $(
                        Self::$name => $value,
                    )*
                    Self::$last_name => $last_value,
                    // Self::Custom(value) => value.as_ref(),
                }
            }
        }
    };
}

mod event;
pub use event::*;

mod tag;
pub use tag::Tag;

mod attribute;
pub use attribute::Attr;


// NOTE: this needs to be above the module definitions because otherwise the
// macro is not declared.
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
            ///// The number of [`Attr`] variants.
            /////
            ///// Useful for things like lookup arrays.
            //// #[cfg(target = "wasm32-unknown-unknown")]
            //pub(crate) const fn variant_count() -> usize {
            //    $last_num + 1
            //}

            /// Convert to numeric representation.
            // #[cfg(target = "wasm32-unknown-unknown")]
            pub(crate) const fn as_usize(self) -> usize {
                self as usize
            }

            // pub(crate) const fn from_u16(value: u16) -> Option<Self> {
            //     match value {
            //         $(
            //             $num => Some(Self::$name),
            //         )*
            //         $last_num => Some(Self::$last_name),
            //         _ => None,
            //     }
            // }

            pub fn as_str(&self) -> &str {
                match self {
                    $(
                        Self::$name => $value,
                    )*
                    Self::$last_name => $last_value,
                    // Self::Custom(value) => value.as_ref(),
                }
            }

            // #[cfg(target = "wasm32-unknown-unknown")]
            pub fn as_js_string(&self) -> &'static js_sys::JsString {
                // #[cfg(target = "wasm32-unknown-unknown")]
                static mut TABLE: $crate::web::StringTable<{$last_num + 1}> = $crate::web::StringTable::new(
                    [
                        $(
                            {
                                $num;
                                (
                                    $value,
                                    once_cell::unsync::OnceCell::new(),
                                )
                            },
                         )*
                        (
                            $last_value,
                            once_cell::unsync::OnceCell::new(),
                        )
                    ]
                );

                // Safety: `static mut` is only unsafe with multi-threading.
                // This is safe because the method is restricted to the
                // wasm32-unknown-unknown target, which does not support
                // multithreading (yet).
                // TODO: add #[cfg] flag to disable otherwise.
                unsafe {
                    TABLE.get(self.as_usize())
                }
            }
        }


        impl From<$enum_name> for $crate::DomStr<'static> {
            fn from(value: $enum_name) -> Self {
                Self::JsStr(value.as_js_string())
            }
        }
    };

}

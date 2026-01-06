macro_rules! define_bitflag {
    (
        $(#[$outer_meta:meta])*
        $vis:vis struct $name:ident: $int:ty {
            $(
                $(#[$inner_meta:meta])*
                $field:ident: $value:expr
            ),*
            $(,)?
        }
    ) => {
        $(#[$outer_meta])*
        $vis struct $name {
            $(
                $(#[$inner_meta])*
                pub $field: bool,
            )*
        }

        const _: () = {
            #[allow(non_upper_case_globals)]
            mod __internal_field_values {
                $(
                    pub const $field: $int = $value as _;
                )*
            }

            impl $name {
                /// ビットフラグから構造体を生成します。
                pub fn from_bits(bits: $int) -> Self {
                    Self {
                        $(
                            $field: (bits & __internal_field_values::$field) != 0,
                        )*
                    }
                }

                /// 構造体をビットフラグに変換します。
                pub fn to_bits(&self) -> $int {
                    let mut bits: $int = 0;
                    $(
                        if self.$field {
                            bits |= __internal_field_values::$field;
                        }
                    )*
                    bits
                }
            }
        };
    };
}

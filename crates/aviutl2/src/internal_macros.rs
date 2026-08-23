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

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let mut parts = Vec::new();
                    $(
                        if self.$field {
                            parts.push(stringify!($field));
                        }
                    )*
                    write!(f, "{}({})", stringify!($name), parts.join(" | "))
                }
            }

            impl std::ops::BitOr for $name {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field | rhs.$field,
                        )*
                    }
                }
            }

            impl std::ops::BitAnd for $name {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field & rhs.$field,
                        )*
                    }
                }
            }

            impl std::ops::Not for $name {
                type Output = Self;

                fn not(self) -> Self::Output {
                    Self {
                        $(
                            $field: !self.$field,
                        )*
                    }
                }
            }

            impl std::ops::BitXor for $name {
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field ^ rhs.$field,
                        )*
                    }
                }
            }

            impl std::ops::Sub for $name {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field & !rhs.$field,
                        )*
                    }
                }
            }
        };
    };
}

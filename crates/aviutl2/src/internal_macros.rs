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
                pub fn from_bits(_bits: $int) -> Self {
                    Self {
                        $(
                            $field: (_bits & __internal_field_values::$field) != 0,
                        )*
                    }
                }

                /// 構造体をビットフラグに変換します。
                pub fn to_bits(&self) -> $int {
                    0 $(| if self.$field { __internal_field_values::$field } else { 0 })*
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let parts: &[(bool, &str)] = &[$((self.$field, stringify!($field))),*];
                    let parts = parts.iter().filter_map(|(enabled, name)| enabled.then_some(*name)).collect::<Vec<_>>();
                    write!(f, "{}({})", stringify!($name), parts.join(" | "))
                }
            }

            impl std::ops::BitOr for $name {
                type Output = Self;

                fn bitor(self, _rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field | _rhs.$field,
                        )*
                    }
                }
            }

            impl std::ops::BitAnd for $name {
                type Output = Self;

                fn bitand(self, _rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field & _rhs.$field,
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

                fn bitxor(self, _rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field ^ _rhs.$field,
                        )*
                    }
                }
            }

            impl std::ops::Sub for $name {
                type Output = Self;

                fn sub(self, _rhs: Self) -> Self::Output {
                    Self {
                        $(
                            $field: self.$field & !_rhs.$field,
                        )*
                    }
                }
            }
        };
    };
}

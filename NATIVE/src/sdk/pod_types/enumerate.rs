pub use paste;

// TODO: EXPERIMENTAL
// TODO: fix DEADCODE AND `Constant Variant2 should have UPPER_SNAKE_CASE`
#[macro_export]
macro_rules! pod_enum {
    (
        $vis:vis enum $name:ident {
            $(
                $variant:ident = $val:expr
            ),* $(,)?
        }
    ) => {
        paste::paste! {

            #[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, PartialEq, Eq)]
            #[repr(transparent)]
            $vis struct $name(pub u8);

            #[allow(non_upper_case_globals, dead_code)]
            impl $name {
                $(
                    pub const $variant: Self = Self($val);
                )*

                #[inline]
                pub fn unpack(&self) -> [<$name View>] {
                    match self.0 {
                        $(
                            $val => [<$name View>]::$variant,
                        )*
                        other => [<$name View>]::Unknown(other),
                    }
                }
            }

            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            $vis enum [<$name View>] {
                $(
                    $variant,
                )*
                Unknown(u8),
            }

            impl From<[<$name View>]> for $name {
                #[inline]
                fn from(view: [<$name View>]) -> Self {
                    match view {
                        $(
                            [<$name View>]::$variant => Self::$variant,
                        )*
                        [<$name View>]::Unknown(val) => Self(val),
                    }
                }
            }
        }
    };
}

mod tests {
    use super::*;
    pod_enum!(
        pub enum TestEnum {
            Variant1 = 1,
            Variant2 = 2,
            Variant3 = 3,
        }
    );

    #[test]
    fn test_pod_enum() {
        let enum_value = TestEnum::Variant2;
        assert_eq!(enum_value.unpack(), TestEnumView::Variant2);
        assert_eq!(TestEnum::from(TestEnumView::Variant2), enum_value);
    }
}

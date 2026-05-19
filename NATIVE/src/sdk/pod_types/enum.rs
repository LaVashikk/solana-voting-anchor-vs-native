// // Твой SDK (my_framework)

// // Экспортируем paste, чтобы юзерам не пришлось его подключать вручную
// pub use paste;

// #[macro_export]
// macro_rules! pod_enum {
//     (
//         // Ожидаем синтаксис: видимость enum Имя { Вариант = Значение, ... }
//         $vis:vis enum $name:ident {
//             $(
//                 $variant:ident = $val:expr
//             ),* $(,)? // Запятая в конце опциональна
//         }
//     ) => {
//         // Используем paste, чтобы склеить имя_View
//         $crate::paste::paste! {

//             // 1. Генерируем POD-структуру для bytemuck
//             #[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, PartialEq, Eq)]
//             #[repr(transparent)]
//             $vis struct $name(pub u8);

//             // 2. Генерируем константы
//             impl $name {
//                 $(
//                     pub const $variant: Self = Self($val);
//                 )*
//             }

//             // 3. Генерируем безопасный Rust-enum для бизнес-логики
//             #[derive(Debug, PartialEq, Eq, Clone, Copy)]
//             $vis enum [<$name View>] {
//                 $(
//                     $variant,
//                 )*
//                 // Обязательный фоллбэк для защиты от мусора из блокчейна
//                 Unknown(u8),
//             }

//             // 4. Связующий код (геттер)
//             impl $name {
//                 #[inline]
//                 pub fn unpack(&self) -> [<$name View>] {
//                     match self.0 {
//                         $(
//                             $val => [<$name View>]::$variant,
//                         )*
//                         other => [<$name View>]::Unknown(other),
//                     }
//                 }
//             }

//             // 5. Удобный сеттер (from)
//             impl From<[<$name View>]> for $name {
//                 #[inline]
//                 fn from(view: [<$name View>]) -> Self {
//                     match view {
//                         $(
//                             [<$name View>]::$variant => Self::$variant,
//                         )*
//                         [<$name View>]::Unknown(val) => Self(val),
//                     }
//                 }
//             }
//         }
//     };
// }

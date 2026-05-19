pub fn string_to_bytes<const N: usize>(s: &str) -> Option<[u8; N]> {
    let mut bytes_arr = [0; N];
    let s_bytes = s.as_bytes();

    if s_bytes.len() > N {
        return None;
    }

    let len = s_bytes.len();
    bytes_arr[..len].copy_from_slice(&s_bytes[..len]);
    Some(bytes_arr)
}

pub fn bytes_to_string<const N: usize>(bytes: &[u8; N]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(N);

    String::from_utf8_lossy(&bytes[..len])
        .trim_matches('\0')
        .to_string()
}

#[macro_export]
macro_rules! require {
    ($cond:expr, $err:expr $(,)?) => {
        if !($cond) {
            ::solana_program::msg!("Error: {}", $err);
            return Err($err.into());
        }
    };
}

#[macro_export]
macro_rules! declare_error {
    ($err_type:ident) => {
        impl From<$err_type> for ::solana_program::program_error::ProgramError {
            fn from(e: $err_type) -> Self {
                ::solana_program::program_error::ProgramError::Custom(e as u32)
            }
        }
    };
}

#[macro_export]
macro_rules! define_program_error {
    ($vis:vis enum $name:ident { $($body:tt)* }) => {
        #[derive(::thiserror::Error, Debug, Copy, Clone, PartialEq)]
        #[cfg_attr(not(target_os = "solana"), derive(::num_enum::TryFromPrimitive))] // todo: what if we dont have num_enum?
        #[repr(u32)]
        $vis enum $name {
            $($body)*
        }

        declare_error!($name);
    }
}


#[macro_export]
macro_rules! route_instructions {
    (
        $program_id:expr, $accounts:expr, $data:expr;

        $($tag:tt => $handler:path),* $(,)?
    ) => {
        if $data.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (header, payload) = $data.split_at(8);
        let tag = u64::from_le_bytes(header.try_into().unwrap());

        match tag {
            $(
                $tag => {
                    ::solana_program::msg!(concat!("Instruction: ", stringify!($tag)));
                    $handler($program_id, $accounts, payload)
                }
            )*
            _ => {
                ::solana_program::msg!("Error: Unknown Instruction Tag");
                Err(::solana_program::program_error::ProgramError::InvalidInstructionData)
            }
        }
    };
}

use bytemuck::{Pod, Zeroable};

use crate::sdk::error::CapacityError;

/// This code is used to verify that FixedString is a multiple of 8 at compile time.
/// It is zero-cost, so it does not affect the binary size,
/// but it prevents mistakes that could lead to memory invariant violations.
pub trait IsMultipleOf8 {}
macro_rules! impl_multiple_of_8 {
    ($($n:expr),*) => {
        $(impl IsMultipleOf8 for [(); $n] {})*
    };
}

// crafty fking shit
impl_multiple_of_8!(
    8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128,
    136, 144, 152, 160, 168, 176, 184, 192, 200, 208, 216, 224, 232, 240, 248, 256
);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FixedString<const N: usize> where [(); N]: IsMultipleOf8 {
    pub(crate) data: [u8; N],
    pub(crate) len: u64,
}

// SAFETY: Zeroable is safe because zeros are valid for both u8 and u64
unsafe impl<const N: usize> Zeroable for FixedString<N> where [(); N]: IsMultipleOf8 {}
// SAFETY: IsMultipleOf8 guarantees the absence of broken alignment.
unsafe impl<const N: usize> Pod for FixedString<N> where [(); N]: IsMultipleOf8 {}

impl<const N: usize> FixedString<N> where [(); N]: IsMultipleOf8 {
    /// Suitable for constants
    /// Panics if the string does not fit
    pub const fn new(s: &str) -> Self {

        let mut bytes_arr = [0; N];
        let s_bytes = s.as_bytes();
        let s_len = s_bytes.len();

        assert!(s_len <= N, "String length exceeds FixedString capacity");

        let mut i = 0;
        while i < s_len {
            bytes_arr[i] = s_bytes[i];
            i += 1;
        }

        Self {
            data: bytes_arr,
            len: s_len as u64,
        }
    }

    /// Safety method for runtime
    pub fn try_new(s: impl AsRef<str>) -> Result<Self, CapacityError> { // todo: use here some kind of CapacityError
        let mut bytes_arr = [0; N];
        let s_bytes = s.as_ref().as_bytes();
        let s_len = s_bytes.len();

        if s_len > N {
            return Err(CapacityError {
                max_capacity: N,
                actual_length: s_len,
            });
        }

        let len = s_len;
        bytes_arr[..len].copy_from_slice(&s_bytes[..len]);

        Ok(Self {
            data: bytes_arr,
            len: s_len as u64,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        // Safety: FixedString implements POD, so an instance can be created bypassing the constructor.
        // Validating checks before casting.
        (self.len as usize).min(N)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.data
    }

    #[inline]
    pub unsafe fn as_str_unchecked(&self) -> &str {
        let safe_len = self.len();
        std::str::from_utf8_unchecked(&self.data[..safe_len])
    }

    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        let safe_len = self.len();
        std::str::from_utf8(&self.data[..safe_len])
    }

    #[inline]
    pub fn as_str_lossy(&self) -> std::borrow::Cow<'_, str> {
        let safe_len = self.len();
        String::from_utf8_lossy(&self.data[..safe_len])
    }
}

impl<const N: usize> TryFrom<&str> for FixedString<N> where [(); N]: IsMultipleOf8 {
    type Error = CapacityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl<const N: usize> TryFrom<String> for FixedString<N> where [(); N]: IsMultipleOf8 {
    type Error = CapacityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

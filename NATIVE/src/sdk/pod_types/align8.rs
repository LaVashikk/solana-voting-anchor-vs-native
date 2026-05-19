use bytemuck::{Pod, Zeroable};

/// Marker trait for types with 8-byte alignment and a size multiple of 8.
///
/// ```
/// // For custom structures:
/// #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// #[repr(C, align(8))]
/// struct MyStruct;
/// native_voter_cheap::unsafe_impl_align8!(MyStruct); // todo: change path
/// ```
pub unsafe trait Align8: Pod + Zeroable {}

#[macro_export]
macro_rules! unsafe_impl_align8 {
    ($t:ty) => {
        const _: () = {
            assert!(::core::mem::size_of::<$t>() % 8 == 0,
                "Type claims Align8, but size_of::<T>() != 8");
        };
        const _:() = {
            assert!(align_of::<$t>() <= 8,
                "Type claims Align8, but align_of::<T>() > 8")
        };
        unsafe impl $crate::sdk::pod_types::align8::Align8 for $t {}
    };
}

unsafe_impl_align8!(i64);
unsafe_impl_align8!(u64);
unsafe_impl_align8!(f64);
unsafe_impl_align8!(solana_program::pubkey::Pubkey); // todo: this is CUSTOM SOLANA type...
unsafe_impl_align8!([u8; 8]);
unsafe_impl_align8!([u8; 16]);
unsafe_impl_align8!([u8; 24]);
unsafe_impl_align8!([u8; 32]);
unsafe impl<const N: usize> Align8 for super::string::FixedString<N> where [(); N]: super::string::IsMultipleOf8 {}

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(transparent)]
pub struct PodBool(u8);

impl PodBool {
    pub const TRUE: Self = Self(1);
    pub const FALSE: Self = Self(0);
    pub fn get(&self) -> bool { self.0 != 0 }
    pub fn set(&mut self, v: bool) { self.0 = v as u8; }
}

impl From<bool> for PodBool {
    fn from(v: bool) -> Self {
        Self(v as u8)
    }
}

impl From<PodBool> for bool {
    fn from(v: PodBool) -> Self {
        v.0 != 0
    }
}

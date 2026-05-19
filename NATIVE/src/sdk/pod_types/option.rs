use bytemuck::{Pod, Zeroable};

use super::align8::Align8;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct PodOption<T: Align8> {
    discriminant: u64, // 0 = None, 1 = Some
    value: T,
}

// CONSTRUCTORS
impl<T: Align8> PodOption<T> {
    pub const fn new(value: T) -> Self {
        Self {
            discriminant: 1,
            value,
        }
    }

    pub const fn none() -> Self {
        Self {
            discriminant: 0,
            value: unsafe { std::mem::zeroed() },
        }
    }

    pub const fn some(value: T) -> Self {
        Self::new(value)
    }
}

// METHODS
impl<T: Align8> PodOption<T> {
    #[inline]
    pub const fn is_some(&self) -> bool {
        self.discriminant == 1
    }

    #[inline]
    pub const fn is_none(&self) -> bool {
        self.discriminant != 1
    }

    #[inline]
    pub const fn as_ref(&self) -> Option<&T> {
        if self.is_some() {
            Some(&self.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        if self.is_some() {
            Some(&mut self.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn take(&mut self) -> Option<T> {
        if self.is_some() {
            self.discriminant = 0;
            Some(self.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn replace(&mut self, value: T) -> Option<T> {
        let old = self.take();
        *self = Self::some(value);
        old
    }

    #[inline]
    pub fn unwrap(self) -> T {
        assert!(self.is_some(), "called `PodOption::unwrap()` on a `None` value");
        self.value
    }

    #[inline]
    pub fn expect(self, msg: &str) -> T {
        assert!(self.is_some(), "{}", msg);
        self.value
    }

    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        if self.is_some() {
            self.value
        } else {
            default
        }
    }

    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        if self.is_some() {
            self.value
        } else {
            T::default()
        }
    }

    #[inline]
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        if self.is_some() {
            self.value
        } else {
            f()
        }
    }

    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        if self.is_some() {
            Some(f(self.value))
        } else {
            None
        }
    }
}

// TODO: safety comment??
unsafe impl<T: Align8> Zeroable for PodOption<T> {
    fn zeroed() -> Self {
        Self::none()
    }
}

// TODO: safety comment??
unsafe impl<T: Align8> Pod for PodOption<T> {}

impl<T: Align8> From<T> for PodOption<T> {
    fn from(value: T) -> Self {
        Self::some(value)
    }
}

impl<T: Align8> From<Option<T>> for PodOption<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => Self::some(v),
            None => Self::none(),
        }
    }
}

impl<T: Align8> From<PodOption<T>> for Option<T> {
    fn from(pod_opt: PodOption<T>) -> Self {
        if pod_opt.is_some() {
            Some(pod_opt.value)
        } else {
            None
        }
    }
}

impl<T: Align8 + std::fmt::Debug> std::fmt::Debug for PodOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            Some(v) => write!(f, "PodSome({:?})", v),
            None => write!(f, "PodNone"),
        }
    }
}

impl<T: Align8 + PartialEq> PartialEq for PodOption<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.discriminant != other.discriminant {
            return false;
        }

        if self.discriminant == 0 {
            return true;
        }

        self.value == other.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_none() {
        let some = PodOption::some(42u64);
        assert!(some.is_some());
        assert_eq!(some.unwrap(), 42);

        let none: PodOption<u64> = PodOption::none();
        assert!(none.is_none());
    }

    #[test]
    fn test_pod_compatible() {
        let opt = PodOption::some(123u64);
        let bytes = bytemuck::bytes_of(&opt);
        let restored: &PodOption<u64> = bytemuck::from_bytes(bytes);
        assert_eq!(restored.unwrap(), 123);
    }

    #[test]
    fn test_zeroable() {
        let zero: PodOption<u64> = Zeroable::zeroed();
        assert!(zero.is_none());
        assert!(!zero.is_some());
    }
}

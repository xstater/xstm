use crate::{versioned_lock::VersionedLock, TVar};

/// TVar without generic T
#[derive(Clone, Copy)]
pub struct AnyTVar<'var> {
    // the pointer to TVar (value.as_ptr())
    // Cannot deref, used in comparing
    pub ptr: *const (),
    // the lock in TVar
    pub lock: &'var VersionedLock,
}

impl<'var> PartialEq for AnyTVar<'var> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr, other.ptr)
    }
}

impl<'var> AnyTVar<'var> {
    pub fn from_var<T: Copy>(var: &'var TVar<T>) -> AnyTVar<'var> {
        AnyTVar {
            ptr: var.value_ptr() as *const _,
            lock: var.get_lock(),
        }
    }
}

impl<'var, T: Copy> From<&'var TVar<T>> for AnyTVar<'var> {
    fn from(value: &'var TVar<T>) -> Self {
        AnyTVar::from_var(value)
    }
}

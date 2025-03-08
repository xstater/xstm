use super::any_var::AnyTVar;
use crate::TVar;

#[cfg(feature = "small_alloc")]
use smallvec::SmallVec;

pub struct ReadSet<'var> {
    #[cfg(not(feature = "small_alloc"))]
    entries: Vec<Entry<'var>>,
    
    #[cfg(feature = "small_alloc")]
    entries: SmallVec<[Entry<'var>; 16]>,
}

pub type Entry<'var> = super::any_var::AnyTVar<'var>;

impl<'var> ReadSet<'var> {
    pub fn new() -> Self {
        ReadSet {
            #[cfg(not(feature = "small_alloc"))]
            entries: Vec::with_capacity(16),

            #[cfg(feature = "small_alloc")]
            entries: SmallVec::new()
        }
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = Entry<'var>> + '_ {
        self.entries.iter().copied()
    }

    fn get_entry<T: Copy, Var: Into<AnyTVar<'var>>>(&self, var: Var) -> Option<Entry<'var>> {
        let var = var.into();
        self.entries
            .iter()
            .find(|entry| *entry == &var)
            .map(|entry| *entry)
    }

    /// Log an read entry
    pub fn log<T: Copy>(&mut self, var: &'var TVar<T>) {
        if let None = self.get_entry::<T, _>(var) {
            // create entry
            self.entries.push(Entry {
                ptr: var.value_ptr() as *const _,
                lock: var.get_lock(),
            });
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

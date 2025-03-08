use crate::{
    version::Version,
    versioned_lock::{self},
    TVar,
};

use super::any_var::AnyTVar;

#[cfg(feature = "small_alloc")]
use smallvec::SmallVec;

/// Write-Set
pub struct WriteSet<'var> {
    #[cfg(not(feature = "small_alloc"))]
    buffer: Vec<u8>,
    #[cfg(not(feature = "small_alloc"))]
    entries: Vec<Entry<'var>>,

    
    #[cfg(feature = "small_alloc")]
    buffer: SmallVec<[u8; 512]>,
    #[cfg(feature = "small_alloc")]
    entries: SmallVec<[Entry<'var>; 16]>,
}

#[derive(Clone, Copy)]
struct Entry<'var> {
    // the var without generic T
    var: AnyTVar<'var>,
    // start byte index in buffer
    offset: usize,
    // the size of T
    len: usize,
}


impl<'var> Entry<'var> {
    // Can only used for write entry
    fn get_ptr_from_buffer(&self, buffer: &[u8]) -> *const u8 {
        let begin = self.offset;
        let end = begin + self.len;
        let slice = &buffer[begin..end];
        slice.as_ptr() as *const u8
    }

    // Can only used for write entry
    fn get_mut_ptr_from_buffer(&self, buffer: &mut [u8]) -> *mut u8 {
        let begin = self.offset;
        let end = begin + self.len;
        let slice = &mut buffer[begin..end];
        slice.as_mut_ptr() as *mut u8
    }
}

impl<'var> WriteSet<'var> {
    pub fn new() -> WriteSet<'var> {
        WriteSet {
            // 512B
            #[cfg(not(feature = "small_alloc"))]
            buffer: Vec::with_capacity(512),
            #[cfg(not(feature = "small_alloc"))]
            entries: Vec::with_capacity(16),

            #[cfg(feature = "small_alloc")]
            buffer: SmallVec::new(),
            #[cfg(feature = "small_alloc")]
            entries: SmallVec::new()
        }
    }

    fn get_entry<T: Copy>(&self, var: &'var TVar<T>) -> Option<Entry<'var>> {
        self.entries
            .iter()
            .find(|entry| &entry.var == &var.into())
            .copied()
    }

    fn get_or_create_entry<T: Copy>(&mut self, var: &'var TVar<T>) -> Entry<'var> {
        if let Some(entry) = self.get_entry(var) {
            entry
        } else {
            // create a new read entry
            
            // allocate buffer
            let offset = self.buffer.len();
            let len = std::mem::size_of::<T>();
            let padding = calculate_padding(len);

            // push len + padding bytes
            let new_len = self.buffer.len() + len + padding;
            self.buffer.resize(new_len, 0);
            
            // create entry
            self.entries.push(Entry {
                var: var.into(),
                offset,
                len,
            });
            *self.entries.last().unwrap_or_else(|| unreachable!())
        }
    }

    /// log an write entry
    pub fn log<T: Copy>(&mut self, var: &'var TVar<T>, value: T) {
        // Get or create entry
        let write_entry = self.get_or_create_entry(var);

        // Copy the data to buffer
        let ptr = write_entry.get_mut_ptr_from_buffer(&mut self.buffer) as *mut T;
        // write to buffer
        unsafe { ptr.write(value) };
    }

    /// read value from logs
    pub fn try_read<T: Copy>(&self, var: &'var TVar<T>) -> Option<T> {
        let entry = self.get_entry(var)?;

        // Read value from write entry
        let ptr = entry.get_ptr_from_buffer(&self.buffer) as *const T;

        Some(unsafe { ptr.read() })
    }

    /// Try to lock all write entries
    pub fn try_lock(&self, retry_count: usize) -> Option<Guard<'_, 'var>> {
        #[cfg(not(feature = "small_alloc"))]
        let mut guards = Vec::with_capacity(self.entries.len());

        #[cfg(feature = "small_alloc")]
        let mut guards = SmallVec::<[_; 16]>::new();

        for entry in self.entries.iter() {
            
            let mut count = 0;

            'spin_loop: loop {
                // spin
                std::hint::spin_loop();

                if let Some(guard) = entry.var.lock.try_lock() {
                    guards.push(GuardedEntry {
                        guard,
                        entry: *entry,
                    });

                    break 'spin_loop;
                }

                if count > retry_count {
                    return None;
                }

                count += 1;
            }
        }

        Some(Guard { guards, buffer: &self.buffer })
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.entries.clear();
    }
}

fn calculate_padding(size: usize) -> usize {
    // align to usize
    size % std::mem::size_of::<usize>()
}

struct GuardedEntry<'var> {
    guard: versioned_lock::Guard<'var>,
    entry: Entry<'var>,
}

pub struct Guard<'write_set, 'var> {
    #[cfg(not(feature = "small_alloc"))]
    guards: Vec<GuardedEntry<'var>>,

    #[cfg(feature = "small_alloc")]
    guards: SmallVec<[GuardedEntry<'var>; 16]>,
    
    buffer: &'write_set [u8]
}

impl<'write_set, 'var> Guard<'write_set, 'var> {
    /// Set all versions of write locks to new version
    pub fn set_version(&mut self, new_version: Version) {
        for guarded_entry in &mut self.guards {
            guarded_entry.guard.set_version(new_version);
        }
    }

    pub fn write_data_from_buffer(&mut self) {
        for guarded_entry in &mut self.guards {
            let buffer_ptr: *const u8 = guarded_entry.entry.get_ptr_from_buffer(self.buffer);

            let cell_ptr = guarded_entry.entry.var.ptr as *mut u8;

            // copy the data in buffer to cell
            unsafe {
                std::ptr::copy_nonoverlapping(buffer_ptr, cell_ptr, guarded_entry.entry.len);
            }
        }
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = AnyTVar<'var>> + '_ {
        self.guards
            .iter()
            .map(|guard| guard.entry.var)
    }
}

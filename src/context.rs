use crate::{version::Version, version_clock::VersionClock, StmError, TVar};

mod readonly;
mod write;

// Hide the details for user
enum ContextInternal<'var> {
    ReadOnly(readonly::Context<'var>),
    Write(write::Context<'var>),
}

pub struct Context<'var> {
    internal: ContextInternal<'var>,
}

// Public methods
impl<'var> Context<'var> {
    pub fn read<T: Copy>(&mut self, var: &'var TVar<T>) -> Result<T, StmError> {
        match &mut self.internal {
            ContextInternal::ReadOnly(context) => context.read(var),
            ContextInternal::Write(context) => context.read(var),
        }
    }

    pub fn write<T: Copy>(&mut self, var: &'var TVar<T>, value: T) -> Result<(), StmError> {
        match &mut self.internal {
            ContextInternal::ReadOnly(context) => context.write(var, value),
            ContextInternal::Write(context) => context.write(var, value),
        }
    }
}

// Internal methods
impl<'var> Context<'var> {
    pub(crate) fn new(read_version: Version) -> Self {
        Context {
            internal: ContextInternal::ReadOnly(readonly::Context::new(read_version)),
        }
    }

    pub(crate) fn reset(&mut self, read_version: Version) {
        match &mut self.internal {
            ContextInternal::ReadOnly(context) => {
                if context.tried_writing() {
                    // Tried to write in read-only context
                    // Convert it to write context
                    self.internal = ContextInternal::Write(write::Context::new(read_version))
                } else {
                    // just reset the read_only context
                    context.reset(read_version);
                }
            }
            ContextInternal::Write(context) => {
                context.reset(read_version);
            }
        }
    }

    pub(crate) fn try_commit(&mut self, clock: &VersionClock) -> Result<(), StmError>{
        match &mut self.internal {
            ContextInternal::ReadOnly(context) => context.try_commit(),
            ContextInternal::Write(context) => context.try_commit(clock),
        }
    }
}

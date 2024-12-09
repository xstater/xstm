use crate::{version::Version, version_clock::VersionClock, StmError, TVar};

mod readonly;
mod write;

// Hide the details for user
enum ContextInteranl<'var> {
    ReadOnly(readonly::Context<'var>),
    Write(write::Context<'var>),
}

pub struct Context<'var> {
    internal: ContextInteranl<'var>,
}

// Public methods
impl<'var> Context<'var> {
    pub fn read<T: Copy>(&mut self, var: &'var TVar<T>) -> Result<T, StmError> {
        match &mut self.internal {
            ContextInteranl::ReadOnly(context) => context.read(var),
            ContextInteranl::Write(context) => context.read(var),
        }
    }

    pub fn write<T: Copy>(&mut self, var: &'var TVar<T>, value: T) -> Result<(), StmError> {
        match &mut self.internal {
            ContextInteranl::ReadOnly(context) => context.write(var, value),
            ContextInteranl::Write(context) => context.write(var, value),
        }
    }
}

// Internal methods
impl<'var> Context<'var> {
    pub(crate) fn new(read_version: Version) -> Self {
        Context {
            internal: ContextInteranl::ReadOnly(readonly::Context::new(read_version)),
        }
    }

    pub(crate) fn reset(&mut self, read_version: Version) {
        match &mut self.internal {
            ContextInteranl::ReadOnly(context) => {
                if context.tried_writing() {
                    // Tried to write in read-only context
                    // Convert it to write context
                    self.internal = ContextInteranl::Write(write::Context::new(read_version))
                } else {
                    // just reset the read_only context
                    context.reset(read_version);
                }
            }
            ContextInteranl::Write(context) => {
                context.reset(read_version);
            }
        }
    }

    pub(crate) fn try_commit(&mut self, clock: &VersionClock) -> Result<(), StmError>{
        match &mut self.internal {
            ContextInteranl::ReadOnly(context) => context.try_commit(),
            ContextInteranl::Write(context) => context.try_commit(clock),
        }
    }
}

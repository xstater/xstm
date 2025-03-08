use crate::{transaction::Transaction, version_clock::VersionClock, Context};

pub struct Stm {
    global_version_clock: VersionClock,
}

impl Stm {
    pub fn new() -> Self {
        Stm {
            global_version_clock: VersionClock::new(),
        }
    }

    pub fn atomically<T: Transaction>(&self, transaction: T) -> T::Output {
        let mut context = Context::new(1.into());
        loop {
            let read_version = self.global_version_clock.sample();

            context.reset(read_version);

            // run transaction
            match transaction.atomically(&mut context) {
                Ok(result) => match context.try_commit(&self.global_version_clock) {
                    Ok(_) => return result,
                    #[cfg(not(feature = "retry_info"))]
                    Err(_) => (),
                    #[cfg(feature = "retry_info")]
                    Err(err) => {
                        let id = std::thread::current().id();
                        println!("Transaction Retried in {:?}: {:?}", id, err)
                    }
                },
                #[cfg(not(feature = "retry_info"))]
                Err(_) => (),
                #[cfg(feature = "retry_info")]
                Err(err) => {
                    let id = std::thread::current().id();
                    println!("Transaction Retried in {:?}: {:?}", id, err)
                }
            }

            // failed and retry
        }
    }
}

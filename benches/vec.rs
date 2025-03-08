// use divan::AllocProfiler;
// #[global_allocator]
// static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

fn thread_counts() -> Vec<usize> {
    vec![0 /* all threads */, 1, 2, 4, 8, 16, 32]
}

const VARS_COUNT: usize = 10;

#[divan::bench_group(threads = thread_counts())]
mod stm {
    use divan::Bencher;
    use xstm::{Context, Stm, StmError, TVar, Transaction};

    use crate::VARS_COUNT;

    struct Vars<'a> {
        vars: &'a [TVar<i32>],
    }

    impl<'a> Transaction for Vars<'a> {
        type Output = ();

        fn atomically<'this: 'var, 'context, 'var>(
            &'this self,
            context: &'context mut Context<'var>,
        ) -> Result<Self::Output, StmError> {
            for var in self.vars {
                let x = context.read(var)?;
                context.write(var, x + 1)?
            }

            Ok(())
        }
    }

    struct Sum<'a> {
        vars: &'a [TVar<i32>],
    }

    impl<'a> Transaction for Sum<'a> {
        type Output = i32;

        fn atomically<'this: 'var, 'context, 'var>(
            &'this self,
            context: &'context mut Context<'var>,
        ) -> Result<Self::Output, StmError> {
            let mut sum = 0;
            for var in self.vars {
                let x = context.read(var)?;
                sum += x;
            }

            Ok(sum)
        }
    }

    #[divan::bench]
    fn write(bencher: Bencher) {
        let vars = std::iter::repeat_n(1, VARS_COUNT)
            .map(|_| TVar::new(1))
            .collect::<Vec<_>>();

        let stm = Stm::new();

        bencher.bench(|| {
            let vars = Vars { vars: &vars };
            stm.atomically(vars);
        });
    }

    #[divan::bench]
    fn read(bencher: Bencher) {
        let vars = std::iter::repeat_n(1, VARS_COUNT)
            .map(|_| TVar::new(1))
            .collect::<Vec<_>>();

        let stm = Stm::new();

        bencher.bench(|| {
            let sum = Sum { vars: &vars };
            stm.atomically(sum);
        });
    }
}

#[divan::bench_group(threads = thread_counts())]
mod mutex {
    use crate::VARS_COUNT;
    use divan::Bencher;
    use std::sync::Mutex;

    #[divan::bench]
    fn write(bencher: Bencher) {
        let locks = std::iter::repeat_n(1, VARS_COUNT)
            .map(|_| Mutex::new(1))
            .collect::<Vec<_>>();

        bencher.bench(|| {
            let guards = locks
                .iter()
                .map(|mutex| mutex.lock().unwrap())
                .collect::<Vec<_>>();

            for mut guard in guards.into_iter() {
                *guard += 1
            }
        });
    }

    #[divan::bench]
    fn read(bencher: Bencher) {
        let locks = std::iter::repeat_n(1, VARS_COUNT)
            .map(|_| Mutex::new(1))
            .collect::<Vec<_>>();

        bencher.bench(|| {
            let guards = locks
                .iter()
                .map(|mutex| mutex.lock().unwrap())
                .collect::<Vec<_>>();

            let mut sum = 0;
            for guard in guards.into_iter() {
                sum += *guard;
            }

            std::hint::black_box(sum);
        });
    }
}

#[divan::bench_group(threads = thread_counts())]
mod rw_lock {
    use std::sync::RwLock;
    use divan::Bencher;
    use crate::VARS_COUNT;

    #[divan::bench]
    fn write(bencher: Bencher) {
        let locks = std::iter::repeat_n(1, VARS_COUNT)
            .map(|_| RwLock::new(1))
            .collect::<Vec<_>>();

        bencher.bench(|| {
            let guards = locks
                .iter()
                .map(|mutex| mutex.write().unwrap())
                .collect::<Vec<_>>();

            for mut guard in guards.into_iter() {
                *guard += 1
            }
        });
    }

    #[divan::bench]
    fn read(bencher: Bencher) {
        let locks = std::iter::repeat_n(1, VARS_COUNT)
            .map(|_| RwLock::new(1))
            .collect::<Vec<_>>();

        bencher.bench(|| {
            let guards = locks
                .iter()
                .map(|mutex| mutex.read().unwrap())
                .collect::<Vec<_>>();

            let mut sum = 0;
            for guard in guards.into_iter() {
                sum += *guard;
            }

            std::hint::black_box(sum);
        });
    }
}

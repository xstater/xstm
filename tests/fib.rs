use std::sync::Arc;
use xstm::{Context, Stm, StmError, TVar, Transaction};

struct Fib {
    var_index: Arc<TVar<i32>>,
    var_a: Arc<TVar<u128>>,
    var_b: Arc<TVar<u128>>,
}

impl Transaction for Fib {
    type Output = (i32, u128);

    fn atomically<'this: 'var, 'context, 'var>(
        &'this self,
        context: &'context mut Context<'var>,
    ) -> Result<Self::Output, StmError> {
        let index = context.read(&self.var_index)?;
        context.write(&self.var_index, index + 1)?;

        let a = context.read(&self.var_a)?;
        let b = context.read(&self.var_b)?;
        let result = a + b;

        context.write(&self.var_a, b)?;
        context.write(&self.var_b, result)?;

        Ok((index, result))
    }
}

#[tokio::test]
async fn basic() {
    let stm = Arc::new(Stm::new());

    let var_index = Arc::new(TVar::new(1));
    let var_a = Arc::new(TVar::new(1));
    let var_b = Arc::new(TVar::new(1));

    let count = 10_000;

    let mut handles = Vec::new();
    for _ in 0..count {
        let _var_index = var_index.clone();
        let _var_a = var_a.clone();
        let _var_b = var_b.clone();
        let _stm = stm.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let fib = Fib {
                var_index: _var_index,
                var_a: _var_a,
                var_b: _var_b,
            };

            let (index, result) = _stm.atomically(fib);

            // let tid = std::thread::current().id();
            // println!("ThreadID {tid:?} index = {index:?} : {result}");

            (index, result)
        });
        handles.push(handle);
    }
    let mut results = Vec::new();

    for handle in handles {
        results.push(handle.await.unwrap());
    }

    results.sort_by_key(|(index, _)| *index);

    let results = results
        .into_iter()
        .map(|(_, result)| result)
        .collect::<Vec<_>>();

    let fib = fib(count);

    let all = results
        .into_iter()
        .zip(fib.into_iter())
        .enumerate()
        .collect::<Vec<_>>();

    for window in all.windows(5) {
        let len = window.len();
        let half_index = len / 2;

        let middle = window[half_index];

        let (index, (result, fib)) = middle;

        if result != fib {
            let indices = window.iter().map(|(index, _)| index).collect::<Vec<_>>();

            let results = window
                .iter()
                .map(|(_, (result, _))| result)
                .collect::<Vec<_>>();

            let fibs = window.iter().map(|(_, (_, fibs))| fibs).collect::<Vec<_>>();

            assert_eq!(
                result, fib,
                "in {}: result({}) != fib({})\n{:?}\n{:?}\n{:?}",
                index, result, fib, &indices, &results, &fibs
            );
            return;
        }
    }
}

fn fib(count: i32) -> Vec<u128> {
    let mut a = 1;
    let mut b = 1;

    let mut v = Vec::new();
    for _ in 0..count {
        let result = a + b;

        a = b;
        b = result;

        v.push(result);
    }
    v
}

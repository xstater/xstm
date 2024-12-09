use std::sync::Arc;

use xstm::{Context, Stm, StmError, TVar, Transaction};

type BigArray = TVar<[u32; 10086]>;

struct Update {
    array: Arc<BigArray>,
}

impl Transaction for Update {
    type Output = u32;

    fn atomically<'this: 'var, 'context, 'var>(
        &'this self,
        context: &'context mut Context<'var>,
    ) -> Result<Self::Output, StmError> {
        let mut array = context.read(&self.array)?;

        for (_ , window) in array.windows(2).enumerate() {
            let first = window[0];
            let second = window[1];

            if second != first + 1 {
                let tid = std::thread::current().id();
                panic!("{:?} Bad Input: {:?}", tid,window);
            }
        }

        for i in &mut array {
            *i += 1
        }

        context.write(&self.array, array)?;

        Ok(array[0])
    }
}

#[tokio::test]
async fn basic() {
    let stm = Arc::new(Stm::new());

    let mut array = [0; 10086];

    let mut gen = 1..;
    array.fill_with(|| gen.next().unwrap());

    let array = Arc::new(TVar::new(array));

    let count = 100;

    let mut handles = Vec::new();
    for _ in 0..count {
        let update_transaction = Update {
            array: array.clone(),
        };
        let _stm = stm.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let first = _stm.atomically(update_transaction);

            let tid = std::thread::current().id();
            println!("ThreadID {tid:?} commited array[0] = {first}");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap()
    }

    let result_array = stm.atomically(array.read());

    let correct_array = generate_correct_array(result_array.len(), count);

    let all = result_array
        .iter()
        .zip(correct_array.iter())
        .enumerate()
        .collect::<Vec<_>>();

    for window in all.windows(5) {
        let len = window.len();
        let half_index = len / 2;

        let middle = window[half_index];

        let (index, (result, expected)) = middle;

        if result != expected {
            let indices = window.iter().map(|(index, _)| index).collect::<Vec<_>>();

            let results = window
                .iter()
                .map(|(_, (result, _))| result)
                .collect::<Vec<_>>();

            let expects = window
                .iter()
                .map(|(_, (_, expects))| expects)
                .collect::<Vec<_>>();

            assert_eq!(
                result, expected,
                "in {}: result({}) != expected({})\nindices: {:?}\nresults: {:?}\nexpects: {:?}",
                index, result, expected, &indices, &results, &expects
            );
            return;
        }
    }
}

fn generate_correct_array(array_len: usize, count: u32) -> Vec<u32> {
    (1_u32..array_len as u32).map(|x| x + count).collect()
}

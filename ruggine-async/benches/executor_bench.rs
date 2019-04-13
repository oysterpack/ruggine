/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]

#[macro_use]
extern crate criterion;

use criterion::Criterion;

use futures::{compat::*, task::SpawnExt};
use ruggine_async::global_executor;

criterion_group!(benches, executor_bench,);

criterion_main!(benches);

/// ## Summary
/// - the futures03 executor is much more efficient and faster than tokio's executor
/// - running tasks on the futures03 executor added ~80 ns overhead
/// - spawning tasks on the futures03 executor added ~290 ns overhead - spawning the tasks overwhelms the
///   CPUs, and as a side effect slows down the overall spawning throughput
///
/// ## Notes
/// - the future needs to do some real work, i.e., not an empty function, because either wise spawning
///   tasks that do nothing overwhelms the executor and results in a panic
/// - LocalPool benchmark was tried, but spawning on a LocalPool in bench test may overwhelm the computer,
///   thus it was removed. That being said, it much more efficient than spawning on a ThreadPool
fn executor_bench(c: &mut Criterion) {
    c.bench_function("ulid_generate_bench", move |b| {
        b.iter(|| {
            let _ = rusty_ulid::Ulid::generate();
        });
    });

    let mut executor = global_executor();
    c.bench_function("executor_spawn_ulid_generate_bench", move |b| {
        b.iter(|| {
            executor
                .spawn(async {
                    let _ = rusty_ulid::Ulid::generate();
                })
                .unwrap();
        });
    });

    let mut executor = global_executor();
    c.bench_function("executor_run_ulid_generate_bench", move |b| {
        b.iter(|| {
            executor.run(async {
                let _ = rusty_ulid::Ulid::generate();
            })
        });
    });

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut executor = rt.executor().compat();
    c.bench_function("executor_tokio_spawn_ulid_generate_bench", move |b| {
        b.iter(|| {
            executor.spawn(async {
                let _ = rusty_ulid::Ulid::generate();
            })
        });
    });
}

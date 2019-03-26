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

//! Provides a global ThreadPool executor.
//!
//! # Global ThreadPool Features
//! - the thread pool is created lazily
//! - uses `ThreadPoolBuilder` defaults for pool size and stack size
//!   - thread prefix name = `ruggine-global-`
//!   - log events:
//!     - debug - after thread is started
//!     - debug - before thread is stopped

use futures::executor::ThreadPool;
use log::debug;

lazy_static::lazy_static! {
    /// Global ThreadPool executor
    /// - uses `ThreadPoolBuilder` defaults for pool size and stack size
    /// - thread prefix name = `ruggine-global-`
    /// - log events:
    ///   - debug - after thread is started
    ///   - debug - before thread is stopped
    static ref GLOBAL_EXECUTOR: ThreadPool = {
        let name_prefix = "ruggine-global";
        let mut builder = ThreadPool::builder();
        builder
            .name_prefix(format!("{}-", name_prefix))
            .after_start(move |thread_index| {
                debug!(
                    "Executer thread has started: {}-{}",
                    name_prefix, thread_index
                )
            })
            .before_stop(move |thread_index| {
                debug!(
                    "Executer thread is stopping: {}-{}",
                    name_prefix, thread_index
                )
            });
        builder.create().expect("Failed to create global ThreadPool")
    };
}

/// returns the global executor
pub fn global_executor() -> ThreadPool {
    GLOBAL_EXECUTOR.clone()
}

#[cfg(test)]
mod tests {
    use futures::task::SpawnExt;
    use log::*;

    fn init_logging() {
        let _ = std::panic::catch_unwind(env_logger::init);
    }

    #[test]
    fn global_executor() {
        init_logging();
        let mut executor = super::global_executor();

        let (tx, rx) = futures::channel::oneshot::channel();
        executor.spawn(async {
            tx.send("CIAO").unwrap();
            super::global_executor().spawn(async {
                let t = std::thread::current();
                info!("{:?}-{:?} : sent msg ...", t.name(), t.id());
            }).unwrap();

        }).unwrap();

        executor.run(async {
            let t = std::thread::current();
            info!("{:?}-{:?} : waiting for msg ...", t.name(), t.id());
            let msg = await!(rx).unwrap();
            super::global_executor().spawn(async move {
                let t = std::thread::current();
                info!("{:?}-{:?} : received msg: {}", t.name(), t.id(), msg);
            }).unwrap();
        });
    }

}

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

//! Futures based executor

use failure::Fail;
use futures::{
    executor::{ThreadPool, ThreadPoolBuilder},
    future::FutureObj,
    prelude::*,
    task::{Spawn, SpawnError},
};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use log::*;
use parking_lot::RwLock;
use rusty_ulid::Ulid;
use std::{fmt, io, num::NonZeroUsize};

lazy_static! {
    /// Global Executor registry
    static ref EXECUTOR_REGISTRY: ExecutorRegistry = ExecutorRegistry::default();
}

///  A general-purpose thread pool based executor for scheduling tasks that poll futures to completion.
/// - The thread pool multiplexes any number of tasks onto a fixed number of worker threads.
#[derive(Debug, Clone)]
pub struct Executor {
    id: ExecutorId,
    threadpool: ThreadPool,
}

impl Executor {
    /// Global ExecutorId, i.e., for the global Executor
    pub const GLOBAL_EXECUTOR_ID: ExecutorId = ExecutorId(1878145360094948138994256176532847089);

    /// returns the ExecutorId
    pub fn id(&self) -> ExecutorId {
        self.id
    }

    /// Runs the given future with this Executor.
    ///
    /// ## Notes
    /// - This function will block the calling thread until the given future is complete.
    /// - The function will return when the provided future completes, even if some of the
    ///   tasks it spawned are still running.
    ///
    /// ## Panics
    /// If the task panics.
    pub fn run<F: Future>(&mut self, f: F) -> F::Output {
        self.threadpool.run(f)
    }
}

impl Spawn for Executor {
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.threadpool.spawn_obj(future)
    }
}

/// Executor unique ID
/// - ULID should be used to gaurantee uniqueness
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ExecutorId(pub u128);

impl ExecutorId {
    /// generates a new unique ULID based ExecutorId
    pub fn generate() -> ExecutorId {
        ExecutorId(Ulid::generate().into())
    }

    /// returns the Ulid representation
    pub fn as_ulid(&self) -> Ulid {
        self.0.into()
    }
}

impl fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ulid: Ulid = self.0.into();
        ulid.fmt(f)
    }
}

impl From<Ulid> for ExecutorId {
    fn from(ulid: Ulid) -> Self {
        Self(ulid.into())
    }
}

/// Executor registry
#[derive(Debug)]
pub struct ExecutorRegistry {
    global_executor: Executor,
    thread_pools: RwLock<HashMap<ExecutorId, Executor>>,
}

impl ExecutorRegistry {
    /// returns the Executor for the specified ID
    pub fn executor(&self, id: ExecutorId) -> Option<Executor> {
        let thread_pools = self.thread_pools.read();
        thread_pools.get(&id).cloned().or_else(|| {
            if id == Executor::GLOBAL_EXECUTOR_ID {
                return Some(self.global_executor.clone());
            }
            None
        })
    }

    /// Returns the global executor
    /// - the thread pool size equals the number of CPU cores.
    pub fn global_executor(&self) -> Executor {
        self.global_executor.clone()
    }

    /// An executor can only be registered once, and once it is registered, it stays registered for the
    /// life of the app.
    fn register(
        &self,
        id: ExecutorId,
        builder: &mut ThreadPoolBuilder,
    ) -> Result<Executor, ExecutorRegistryError> {
        let mut thread_pools = self.thread_pools.write();
        if thread_pools.contains_key(&id) || id == Executor::GLOBAL_EXECUTOR_ID {
            return Err(ExecutorRegistryError::ExecutorAlreadyRegistered(id));
        }
        let executor = Executor {
            id,
            threadpool: builder
                .create()
                .map_err(|err| ExecutorRegistryError::ThreadPoolCreateFailed(err))?,
        };
        thread_pools.insert(id, executor.clone());
        Ok(executor)
    }
}

impl Default for ExecutorRegistry {
    fn default() -> Self {
        let global_executor = {
            let mut builder = ExecutorBuilder::new(Executor::GLOBAL_EXECUTOR_ID).builder();
            Executor {
                id: Executor::GLOBAL_EXECUTOR_ID,
                threadpool: builder
                    .create()
                    .expect("Failed to create global ThreadPool executor"),
            }
        };

        Self {
            global_executor,
            thread_pools: RwLock::new(HashMap::new()),
        }
    }
}

/// Executor registry related errors
#[derive(Fail, Debug)]
pub enum ExecutorRegistryError {
    /// When a ThreadPool creation failure occurs.
    #[fail(display = "Failed to create ThreadPool: {}", _0)]
    ThreadPoolCreateFailed(#[cause] io::Error),
    /// When trying to register an Executor using an ID that is already registered.
    #[fail(display = "Executor is already registered: {}", _0)]
    ExecutorAlreadyRegistered(ExecutorId),
}

/// Executor builder, which is used to register the Executor with the global Executor registry
///
/// ## Example
/// ``` rust
/// # use ruggine_concurrency::executor::*;
/// const EXECUTOR_ID: ExecutorId = ExecutorId(1872692872983539779132843447162269015);
/// let mut executor = ExecutorBuilder::new(EXECUTOR_ID).register().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ExecutorBuilder {
    id: ExecutorId,
    stack_size: Option<NonZeroUsize>,
    pool_size: Option<NonZeroUsize>,
}

impl ExecutorBuilder {
    /// constructor
    /// - with catch_unwind = true
    pub fn new(id: ExecutorId) -> Self {
        Self {
            id,
            stack_size: None,
            pool_size: None,
        }
    }

    /// Sets the thread stack size
    pub fn set_stack_size(mut self, size: NonZeroUsize) -> Self {
        self.stack_size = Some(size);
        self
    }

    /// Sets the thread pool size
    pub fn set_pool_size(mut self, size: NonZeroUsize) -> Self {
        self.pool_size = Some(size);
        self
    }

    fn builder(&self) -> ThreadPoolBuilder {
        let mut builder = ThreadPool::builder();
        let executor_id = self.id;
        builder
            .name_prefix(format!("{}-", executor_id))
            .after_start(move |thread_index| {
                debug!(
                    "Executer thread has started: {}-{}",
                    executor_id, thread_index
                )
            })
            .before_stop(move |thread_index| {
                debug!(
                    "Executer thread is stopping: {}-{}",
                    executor_id, thread_index
                )
            });
        if let Some(ref size) = self.stack_size {
            builder.stack_size(size.get());
        }
        if let Some(ref size) = self.pool_size {
            builder.pool_size(size.get());
        }
        builder
    }

    /// Tries to build and register the Executor with the global ExecutorRegistry.
    ///
    /// An executor can only be registered once, and once it is registered, it stays registered for
    /// the life of the app.
    pub fn register(self) -> Result<Executor, ExecutorRegistryError> {
        let mut threadpool_builder = self.builder();
        EXECUTOR_REGISTRY.register(self.id, &mut threadpool_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

}

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

//! Timer based extensions
//!
//! ## Features
//! - delayed events - [delay()](fn.delay.html)
//! - timeouts can be specified on a future - [FutureTimerExt::timeout()](trait.FutureTimerExt.html#tymethod.timeout)

use crate::FutureResult;
use futures::{compat::*, prelude::*, task::{
    Waker, Poll
}};
use log::*;
use std::{
    sync::mpsc,
    time::{Duration, Instant},
    pin::Pin
};
use failure::Fail;

lazy_static::lazy_static! {

    static ref TIMER_HANDLE: tokio_timer::timer::Handle = {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut timer = tokio_timer::timer::Timer::default();
            tx.send(timer.handle()).unwrap();
            loop {
                if let Err(err) = timer.turn(None) {
                    error!("timer.turn() failed: {:?}", err);
                }
                debug!("timer wheel turned");
            }
        });
        rx.recv().unwrap()
    };

}

/// A future that completes at a specified instant in time.
pub fn delay(duration: Duration) -> tokio_timer::Delay {
    TIMER_HANDLE.delay(Instant::now() + duration)
}

/// Defines timer based extensions for futures
pub trait FutureTimerExt: Future {
    /// Allows a Future to execute for a limited amount of time.
    /// - If the future completes before the timeout has expired, then the completed value is returned.
    ///   Otherwise, a timeout error is returned
    ///
    /// ## Example
    /// ```
    /// # #![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
    /// # use std::time::*;
    /// use futures::{ compat::*, prelude::* };
    /// use ruggine_async::{timer::*, global_executor};
    /// let f = async {
    ///    // simulate spending time doing work
    ///    let sleep = delay(Duration::from_millis(100)).compat();
    ///    let result = await!(sleep);
    ///    Result::<(), ruggine_errors::Never>::Ok(())
    ///  };
    ///  // use the FutureTimerExt::timeout() extension
    ///  let f = f.boxed().timeout(Duration::from_millis(10));
    ///  // the future will sleep for 100 ms, but has a timeout of 10 ms - thus, we expect a timeout error
    ///  if let Err(err) = global_executor().run(f) {
    ///     println!("As expected the task timed out: {}", err);
    ///  } else {
    ///     panic!("The task should have timed out");
    ///  }
    /// ```
    fn timeout(self, timeout: Duration) -> FutureResult<Result<Self::Output, TimeoutError>>;
}

impl<T> FutureTimerExt for T
where
    T: Future + TryFuture + Send + std::marker::Unpin + 'static,
{
    fn timeout(self, timeout: Duration) -> FutureResult<Result<Self::Output, TimeoutError>> {
        let mut delay = delay(timeout).compat().fuse();
        let mut f = self.fuse();

        async move {
            futures::select! {
                result   = f     => Ok(result),
                _timeout = delay =>  Err(TimeoutError(timeout))
            }
        }
            .boxed()
    }
}

/// Timeout error
#[derive(Debug, Fail, Copy, Clone)]
#[fail(display = "Timeout error: {:?}", _0)]
pub struct TimeoutError(Duration);

impl TimeoutError {

    /// Returns the timeout duration
    pub fn timeout(&self) -> Duration {
        self.0
    }
}

/// A stream representing notifications at fixed interval
#[derive(Debug)]
pub struct Interval {
    delay: Compat01As03<tokio_timer::Delay>,
    interval: Duration
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(
        self: Pin<&mut Self>,
        waker: &Waker,
    ) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn init_logging() {
        let _ = std::panic::catch_unwind(env_logger::init);
    }

    #[test]
    fn timeout_completed() {
        init_logging();

        for i in 1..=5 {
            let f = async {
                info!("timeout_completed(): pronto");
                Result::<(), ruggine_errors::Never>::Ok(())
            };

            let future = f.boxed().timeout(Duration::from_millis(50));
            let now = Instant::now();
            let result = crate::global_executor().run(future);
            info!(
                "[{}] timeout_completed(): future result: {:?} : {:?}",
                i,
                result,
                now.elapsed()
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn timeout_timed_out() {
        init_logging();
        for i in 1..=5 {
            let f = async {
                info!("timeout_timed_out() - START");
                let sleep = delay(Duration::from_millis(100)).compat();
                let now = Instant::now();
                let result = await!(sleep);
                info!(
                    "{:?}: timeout_timed_out(): pronto: {:?}",
                    result,
                    now.elapsed()
                );
                Result::<(), ruggine_errors::Never>::Ok(())
            };

            let f = f.boxed().timeout(Duration::from_millis(10));
            let now = Instant::now();
            let result = crate::global_executor().run(f);
            info!(
                "[{}] timeout_timed_out(): future result: {:?} : {:?}",
                i,
                result,
                now.elapsed()
            );
            assert!(result.is_err());
        }
    }
}

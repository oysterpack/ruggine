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

//! Timer based extensions built on top of [tokio-timer](https://crates.io/crates/tokio-timer).
//!
//! ## Notes
//! - [Timer](https://docs.rs/tokio-timer/latest/tokio_timer/timer/struct.Timer.html) has a resolution
//!   of one millisecond. Any unit of time that falls between milliseconds are rounded up to the next millisecond.

use crate::PinnedFuture;
use failure::Fail;
use futures::{
    compat::*,
    prelude::*,
    task::{Poll, Waker},
};
use log::*;
use std::{
    pin::Pin,
    sync::mpsc,
    time::{Duration, Instant},
};

lazy_static::lazy_static! {

    static ref TIMER_HANDLE: tokio_timer::timer::Handle = {
        let (tx, rx) = mpsc::channel();
        // turn the Timer's wheel on a background thread continuously
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

/// Returns the handle for the global [Timer](https://docs.rs/tokio-timer/latest/tokio_timer/timer/struct.Timer.html).
pub fn global_timer_handle() -> &'static tokio_timer::timer::Handle {
    &TIMER_HANDLE
}

/// A future that completes at a specified instant in time. It uses a global [Timer](https://docs.rs/tokio-timer/latest/tokio_timer/timer/struct.Timer.html).
///
/// Only millisecond level resolution is guaranteed. There is no guarantee as to how the sub-millisecond portion of deadline will be handled.
/// This should not be used for high-resolution timer use cases.
///
/// If delay less than 1 ms is requested, then a delay duration of 1 ms will be returned.
///
/// ## NOTES
/// - It appears that 1 ms overhead is added for each delay event. To compensate, 1 ms is subtracted.
///   If after subtracting, the delay is less than 1 ms, than 1 ms is used as the delay.
/// - `tokio_timer::Delay` is a v0.1 Future, but it can easily be converted into a v0.3 future via:
/// ```rust
/// # #![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
/// # use std::time::*;
/// use futures::{ compat::*, prelude::* };
/// use ruggine_async::{timer::*, global_executor};
/// global_executor().run(async {
///    // convert the tokio_timer::Delay into a v3 Future via the futures::compat::Future01CompatExt
///    let sleep = delay(Duration::from_millis(100)).compat();
///    let _ = await!(sleep);
///  });
/// ```
pub fn delay(duration: Duration) -> tokio_timer::Delay {
    delay_with_timer(duration, &TIMER_HANDLE)
}

/// A future that completes at a specified instant in time that is backed by the specified timer.
///
/// Only millisecond level resolution is guaranteed. There is no guarantee as to how the sub-millisecond portion of deadline will be handled.
/// This should not be used for high-resolution timer use cases.
///
/// If delay less than 1 ms is requested, then a delay duration of 1 ms will be returned.
///
/// ## NOTES
/// - It appears that 1 ms overhead is added for each delay event. To compensate, 1 ms is subtracted.
///   If after subtracting, the delay is less than 1 ms, than 1 ms is used as the delay.
/// - `tokio_timer::Delay` is a v0.1 Future, but it can easily be converted into a v0.3 future via:
/// ```rust
/// # #![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
/// # use std::time::*;
/// use futures::{ compat::*, prelude::* };
/// use ruggine_async::{timer::*, global_executor};
/// global_executor().run(async {
///    // convert the tokio_timer::Delay into a v3 Future via the futures::compat::Future01CompatExt
///    let sleep = delay_with_timer(Duration::from_millis(100), global_timer_handle()).compat();
///    let _ = await!(sleep);
///  });
/// ```
pub fn delay_with_timer(
    duration: Duration,
    timer_handle: &tokio_timer::timer::Handle,
) -> tokio_timer::Delay {
    let one_ms = min_duration();
    let duration = duration
        .checked_sub(one_ms)
        .or_else(|| Some(one_ms))
        .unwrap();
    let duration = if duration < one_ms { one_ms } else { duration };
    timer_handle.delay(Instant::now() + duration)
}

/// minimum supported timer duration
pub fn min_duration() -> Duration {
    Duration::from_millis(1)
}

/// Defines timer based extensions for futures
pub trait FutureTimerExt: Future {
    /// Allows a Future to execute for a limited amount of time.
    /// - If the future completes before the timeout has expired, then the completed value is returned.
    ///   Otherwise, a timeout error is returned
    /// - minininum timeout duration is 1 ms
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
    fn timeout(self, timeout: Duration) -> PinnedFuture<Result<Self::Output, TimeoutError>>;
}

impl<T> FutureTimerExt for T
where
    T: Future + TryFuture + Send + std::marker::Unpin + 'static,
{
    fn timeout(self, timeout: Duration) -> PinnedFuture<Result<Self::Output, TimeoutError>> {
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

/// A stream representing notifications at fixed interval.
/// - the minimum supported interval duration is 1 ms. If anything less than 1 ms is specified,
///   then 1 ms will be used.
///
/// ## Example
/// ```
/// # #![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
/// # use std::time::*;
/// # use log::*;
/// use futures::{ compat::*, prelude::* };
/// use ruggine_async::{timer::*, global_executor};
/// global_executor().run( async {
///    let mut interval = Interval::new(Duration::from_millis(5)).take(10);
///    let mut i: usize = 1;
///    let start = Instant::now();
///    while let Some(_) = await!(interval.next()) {
///       info!("interval event #{}: {:?}", i, start.elapsed());
///       i += 1;
///    }
/// });
/// ```
#[derive(Debug)]
pub struct Interval {
    delay: Compat01As03<tokio_timer::Delay>,
    interval: Duration,
}

impl Interval {
    /// Returns the interval duration
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Creates a new Interval that yields at the specified interval duration.
    /// - The minimum supported duration is 1 ms.
    pub fn new(interval: Duration) -> Self {
        Interval::with_timer(interval, global_timer_handle())
    }

    /// Creates a new Interval that yields at the specified interval duration that is backed by the specified timer.
    /// - The minimum supported duration is 1 ms.
    pub fn with_timer(interval: Duration, timer: &tokio_timer::timer::Handle) -> Self {
        Self {
            delay: delay_with_timer(interval, timer).compat(),
            interval,
        }
    }

    /// Create a new Interval that starts after the initial delay and yields every duration interval after that.
    /// - The minimum supported interval duration is 1 ms.
    /// - The initial delay may be a zero duration, which would be the same [Interval::new()](#method.new).
    ///   Otherwise, it must be at least 1 ms.
    pub fn starting_after(initial_delay: Duration, interval: Duration) -> Self {
        Interval::starting_after_with_timer(initial_delay, interval, global_timer_handle())
    }

    /// Create a new Interval that starts after the initial delay and yields every duration interval after that.
    /// - The minimum supported interval duration is 1 ms.
    /// - The initial delay may be a zero duration, which would be the same [Interval::new()](#method.new).
    ///   Otherwise, it must be at least 1 ms.
    pub fn starting_after_with_timer(
        initial_delay: Duration,
        interval: Duration,
        timer: &tokio_timer::timer::Handle,
    ) -> Self {
        if initial_delay == Duration::new(0, 0) {
            return Interval::new(interval);
        }
        Self {
            delay: delay_with_timer(initial_delay, timer).compat(),
            interval,
        }
    }
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Option<Self::Item>> {
        let f = Pin::new(&mut self.delay);
        match f.poll(waker) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(_)) => {
                self.delay = delay(self.interval).compat();
                Poll::Ready(Some(()))
            }
            Poll::Ready(Err(err)) => {
                if err.is_shutdown() {
                    Poll::Ready(None)
                } else {
                    warn!("timer is at capacity: {}", err);
                    Poll::Pending
                }
            }
        }
    }
}

/// Interval error
#[derive(Debug, Fail, Eq, PartialEq, Clone, Copy)]
pub enum IntervalError {
    /// The specified duration is below what is supported
    #[fail(
        display = "The requested duration of {:?} is below the minimum supported duration of 1 ms",
        _0
    )]
    DurationTooSmall(Duration),
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

    #[test]
    fn interval_starting_now() {
        init_logging();

        for i in 0..=10 {
            info!("interval_starting_now(): interval = {} ms", i);
            crate::global_executor().run(
                async {
                    let duration = Duration::from_millis(i);
                    let mut interval = Interval::new(duration).take(3);
                    let mut i: usize = 1;
                    let start = Instant::now();
                    while let Some(_) = await!(interval.next()) {
                        let elapsed_duration = start.elapsed();
                        info!("interval event #{}: {:?}", i, elapsed_duration);
                        i += 1;
                    }
                    let tot_elapsed_duration = start.elapsed();
                    info!(
                        "interval_starting_now(): total time elapsed: {:?}",
                        tot_elapsed_duration
                    );
                },
            );
        }

        let tot_elapsed_duration = crate::global_executor().run(
            async {
                let duration = Duration::from_millis(5);
                let mut interval = Interval::new(duration).take(10);
                let mut i: usize = 1;
                let start = Instant::now();
                while let Some(_) = await!(interval.next()) {
                    let elapsed_duration = start.elapsed();
                    info!("interval event #{}: {:?}", i, elapsed_duration);
                    i += 1;
                }
                let tot_elapsed_duration = start.elapsed();
                info!(
                    "interval_starting_now(): total time elapsed: {:?}",
                    tot_elapsed_duration
                );
                tot_elapsed_duration
            },
        );
        assert!(
            tot_elapsed_duration >= Duration::from_millis(49)
                && tot_elapsed_duration <= Duration::from_millis(51)
        );
    }

    #[test]
    fn interval_starting_after() {
        init_logging();

        for i in 0..=10 {
            info!("interval_starting_now(): interval = {} ms", i);
            crate::global_executor().run(
                async {
                    let duration = Duration::from_millis(i);
                    let mut interval = Interval::starting_after(duration, duration).take(3);
                    let mut i: usize = 1;
                    let start = Instant::now();
                    while let Some(_) = await!(interval.next()) {
                        let elapsed_duration = start.elapsed();
                        info!("interval event #{}: {:?}", i, elapsed_duration);
                        i += 1;
                    }
                    let tot_elapsed_duration = start.elapsed();
                    info!(
                        "interval_starting_at(): total time elapsed: {:?}",
                        tot_elapsed_duration
                    );
                },
            );
        }

        let tot_elapsed_duration = crate::global_executor().run(
            async {
                let duration = Duration::from_millis(5);
                let initial_delay = Duration::from_millis(10);
                let mut interval = Interval::starting_after(initial_delay, duration).take(10);
                let mut i: usize = 1;
                let start = Instant::now();
                while let Some(_) = await!(interval.next()) {
                    let elapsed_duration = start.elapsed();
                    info!("interval event #{}: {:?}", i, elapsed_duration);
                    i += 1;
                }
                let tot_elapsed_duration = start.elapsed();
                info!(
                    "interval_starting_now(): total time elapsed: {:?}",
                    tot_elapsed_duration
                );
                tot_elapsed_duration
            },
        );
        // the first event occurs after 10 ms, and then every 5 seconds there after = 10 ms + (5 ms * 9) = 55 ms
        assert!(
            tot_elapsed_duration >= Duration::from_millis(54)
                && tot_elapsed_duration <= Duration::from_millis(56)
        );
    }
}

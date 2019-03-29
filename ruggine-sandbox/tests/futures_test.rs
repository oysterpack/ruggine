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

use log::*;
use std::{
    panic::catch_unwind,
    time::{Duration, Instant},
};
use tokio::{prelude::*, timer::Delay};

#[test]
fn delay_futures01() {
    let _ = catch_unwind(env_logger::init);
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let delay = Duration::from_millis(100);
    let now = Instant::now();
    let delay = Delay::new(now + delay);
    let result = rt.block_on(delay);
    info!("{:?}: elapsed time = {:?}", result, now.elapsed());
}

#[test]
fn delay_futures03() {
    use ruggine_async::futures::compat::*;
    let _ = catch_unwind(env_logger::init);
    let mut executor = ruggine_async::global_executor();
    let delay = Duration::from_millis(100);
    let now = Instant::now();
    let delay = Delay::new(now + delay);
    let delay = delay.compat();
    let result = executor.run(delay);
    info!("{:?}: elapsed time = {:?}", result, now.elapsed());
}

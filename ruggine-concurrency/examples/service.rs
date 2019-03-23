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
use failure::Fail;
use futures::{executor::ThreadPool, prelude::*, task::SpawnExt};
use ruggine_concurrency::{FutureResult, Service};

#[derive(Debug, Copy, Clone)]
struct Foo;

impl Service<String> for Foo {
    type Response = String;
    type Error = Error;

    fn process(&mut self, req: String) -> FutureResult<String, Error> {
        async move { Ok(format!("request: {}", req)) }.boxed()
    }
}

impl Service<(usize, usize)> for Foo {
    type Response = usize;
    type Error = Error;
    fn process(&mut self, req: (usize, usize)) -> FutureResult<usize, Error> {
        async move { Ok(req.0 + req.1) }.boxed()
    }
}

#[allow(warnings)]
#[derive(Debug, Copy, Clone, Fail)]
enum Error {
    #[fail(display = "Invalid request")]
    InvalidRequest,
}

fn main() -> Result<(), failure::Error> {
    let mut executor = ThreadPool::new().unwrap();
    let (tx, rx) = futures::channel::oneshot::channel();
    executor
        .spawn(
            async move {
                let mut foo = Foo;
                let sum = await!(foo.process((1, 2))).unwrap();
                let msg = await!(foo.process(sum.to_string())).unwrap();
                tx.send(msg).expect("Failed to send response");
            },
        )
        .expect("Failed to spawn task");
    let rep = executor.run(rx)?;
    println!("rep = {:?}", rep);
    Ok(())
}

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

//! Service is the classic concurrency abstraction for async functions.
//!
//! It's design is very much inspired from [tower_service::Service](https://docs.rs/tower-service/latest/tower_service/trait.Service.html).
//!
//! The beauty of this design is that the service is decoupled from the underlying protocol.
//! - The service may be local or remote.
//! - Smart clients can be implemented, i.e., request validation can be run client side before the request is
//!   submitted for async processing, which may result remote network calls.
//! - It also simplifies testing by making it easy to mock services.
//!

use futures::prelude::*;
use std::pin::Pin;

/// Pinned Boxed Future type alias which produces a Result
pub type FutureResult<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

/// Future based service
pub trait Service<Req>
where
    Req: Send,
{
    /// Response type
    type Response: Send;

    /// Error type
    type Error: Send;

    /// Process the request and return the response asynchronously.
    fn process(&mut self, req: Req) -> FutureResult<Self::Response, Self::Error>;
}

#[cfg(test)]
mod test {

    use super::*;
    use futures::{
        executor::ThreadPool,
        task::SpawnExt
    };

    #[derive(Debug, Copy, Clone)]
    struct Foo;

    impl Service<String> for Foo {
        /// Response type
        type Response = String;

        /// Error type
        type Error = Error;

        fn process(&mut self, req: String) -> FutureResult<String, Error> {
            async move { Ok(format!("{}.len() = {}", req, req.len())) }.boxed()
        }
    }

    impl Service<(usize, usize)> for Foo {
        /// Response type
        type Response = usize;

        /// Error type
        type Error = Error;

        fn process(&mut self, req: (usize, usize)) -> FutureResult<usize, Error> {
            async move { Ok(req.0 + req.1) }.boxed()
        }
    }

    #[allow(warnings)]
    #[derive(Debug, Copy, Clone)]
    enum Error {
        InvalidRequest,
    }

    #[test]
    fn service_poc() {
        let mut foo = Foo;
        let rep = foo.process("ciao".to_owned());
        let mut executor = ThreadPool::new().unwrap();
        let rep = executor.run(rep).unwrap();
        println!("rep = {:?}", rep);

        let rep = foo.process((1, 2));
        let rep = executor.run(rep).unwrap();
        println!("rep = {:?}", rep);

        // fully qualified syntax
        let rep = <Foo as Service<(usize, usize)>>::process(&mut foo, (1, 2));
        let rep = executor.run(rep).unwrap();
        println!("rep = {:?}", rep);

        let (tx, rx) = futures::channel::oneshot::channel();
        executor.spawn(async move {
            let sum = await!(foo.process((1,2))).unwrap();
            let msg = await!(foo.process(sum.to_string())).unwrap();
            tx.send(msg).expect("Failed to send response");
        }).expect("Failed to spawn task");
        let rep = executor.run(rx).unwrap();
        println!("rep = {:?}", rep);
    }
}

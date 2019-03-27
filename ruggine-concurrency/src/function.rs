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

//! AsyncFunction is the classic concurrency abstraction.
//!
//! The beauty of this design is that the async function is decoupled from the underlying protocol.
//! - The async function call may be local or remote.
//! - Smart clients can be implemented, i.e., request validation can be run client side before the request is
//!   submitted for async processing, which may result remote network calls.
//! - It also simplifies testing by making it easy to mock services.
//! - It enables code generation for server side stubs and clients.
//!

use futures::prelude::*;
use std::pin::Pin;

/// Pinned Boxed Future type alias which produces a Result
pub type FutureResult<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Future based async function
pub trait AsyncFunction<Req, Rep>
where
    Req: Send,
    Rep: Send,
{
    /// Future result
    type Future: Future<Output = Rep> + Send;

    /// Process the request and return the response asynchronously.
    fn apply(&mut self, req: Req) -> Self::Future;
}

#[cfg(test)]
mod test {

    use super::*;
    use failure::Fail;
    use futures::{executor::ThreadPool, task::SpawnExt};

    #[derive(Debug, Copy, Clone)]
    struct Foo;

    impl AsyncFunction<String, Result<String, Error>> for Foo {
        type Future = FutureResult<Result<String, Error>>;

        fn apply(&mut self, req: String) -> Self::Future {
            async move { Ok(format!("request: {}", req)) }.boxed()
        }
    }

    impl AsyncFunction<(usize, usize), Result<usize, Error>> for Foo {
        type Future = FutureResult<Result<usize, Error>>;

        fn apply(&mut self, req: (usize, usize)) -> Self::Future {
            async move { Ok(req.0 + req.1) }.boxed()
        }
    }

    impl AsyncFunction<String, usize> for Foo {
        type Future = FutureResult<usize>;

        fn apply(&mut self, req: String) -> Self::Future {
            async move { req.len() }.boxed()
        }
    }

    #[allow(warnings)]
    #[derive(Debug, Copy, Clone, Fail)]
    enum Error {
        #[fail(display = "Invalid request")]
        InvalidRequest,
    }

    #[test]
    fn service_poc() {
        let mut foo = Foo;
        // fully qualified syntax is required because the function is overloaded and the compiler does
        // not know which implementation to invoke
        let rep = <Foo as AsyncFunction<String, Result<String, Error>>>::apply(
            &mut foo,
            "ciao".to_owned(),
        );
        let mut executor = ThreadPool::new().unwrap();
        let rep = executor.run(rep).unwrap();
        println!("rep = {:?}", rep);

        let rep = foo.apply((1, 2));
        let rep = executor.run(rep).unwrap();
        println!("rep = {:?}", rep);

        let rep = foo.apply((1, 2));
        let rep = executor.run(rep).unwrap();
        println!("rep = {:?}", rep);

        let (tx, rx) = futures::channel::oneshot::channel();
        executor
            .spawn(
                async move {
                    let sum = await!(foo.apply((1, 2))).unwrap();
                    let msg = await!(<Foo as AsyncFunction<String, usize>>::apply(
                        &mut foo,
                        sum.to_string()
                    ));
                    tx.send(msg).expect("Failed to send response");
                },
            )
            .expect("Failed to spawn task");
        let rep = executor.run(rx).unwrap();
        println!("rep = {:?}", rep);
        assert_eq!(rep, 1);
    }
}

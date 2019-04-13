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

//! Asynchronous functions modeled after gRPC
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

/// Pinned Boxed Future
pub type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Pinned Boxed Stream
pub type PinnedStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

/// Unary function where the client sends a single request to the server and gets a single
/// response future back, just like a normal function call.
pub trait Unary<Req, Rep> {
    /// Future result
    type Future: Future<Output = Rep> + Send;

    /// Process the request and return the response asynchronously.
    fn apply(&mut self, req: Req) -> Self::Future;
}

/// Client streaming function where the client writes a sequence of messages and sends them to the server,
/// again using a provided stream. Once the client has finished writing the messages, it waits for
/// the server to read them and return its response.
pub trait ClientStreaming<Req, Rep> {
    /// Request stream
    type Stream: Stream<Item = Req>;

    /// Future result
    type Future: Future<Output = Rep> + Send;

    /// Process the request stream and return the response asynchronously.
    fn apply(&mut self, req: Self::Stream) -> Self::Future;
}

/// Server streaming function where the client sends a request to the server and gets a stream to
/// read a sequence of messages back. The client reads from the returned stream until there are no
/// more messages.
pub trait ServerStreaming<Req, Rep> {
    /// Response stream
    type Stream: Stream<Item = Rep>;

    /// Process the request and stream response messages asynchronously.
    fn apply(&mut self, req: Req) -> Self::Stream;
}

/// Bidirectional streaming function where both sides send a sequence of messages using a read-write stream.
/// The two streams operate independently, so clients and servers can read and write in whatever order they like:
/// for example, the server could wait to receive all the client messages before writing its responses,
/// or it could alternately read a message then write a message, or some other combination of reads and writes.
pub trait BidirectionalStreaming<Req, Rep> {
    /// Response stream
    type ReqStream: Stream<Item = Req>;

    /// Response stream
    type RepStream: Stream<Item = Rep>;

    /// Process the request and stream response messages asynchronously.
    fn apply(&mut self, req: Self::ReqStream) -> Self::RepStream;
}

#[cfg(test)]
mod test {

    use super::*;
    use failure::Fail;
    use futures::task::SpawnExt;
    use log::*;

    fn init_logging() {
        let _ = std::panic::catch_unwind(env_logger::init);
    }

    #[test]
    fn unary() {
        #[derive(Debug, Copy, Clone)]
        struct Foo;

        impl Unary<String, Result<String, Error>> for Foo {
            type Future = PinnedFuture<Result<String, Error>>;

            fn apply(&mut self, req: String) -> Self::Future {
                async move { Ok(format!("request: {}", req)) }.boxed()
            }
        }

        impl Unary<(usize, usize), Result<usize, Error>> for Foo {
            type Future = PinnedFuture<Result<usize, Error>>;

            fn apply(&mut self, req: (usize, usize)) -> Self::Future {
                async move { Ok(req.0 + req.1) }.boxed()
            }
        }

        impl Unary<String, usize> for Foo {
            type Future = PinnedFuture<usize>;

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

        init_logging();

        let mut foo = Foo;
        // fully qualified syntax is required because the function is overloaded and the compiler does
        // not know which implementation to invoke
        let rep = <Foo as Unary<String, Result<String, Error>>>::apply(&mut foo, "ciao".to_owned());
        let mut executor = crate::global_executor();
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
            .spawn(async move {
                let sum = await!(foo.apply((1_usize, 2_usize))).unwrap();
                let msg = await!(<Foo as Unary<String, usize>>::apply(
                    &mut foo,
                    sum.to_string()
                ));
                tx.send(msg).expect("Failed to send response");
            })
            .expect("Failed to spawn task");
        let rep = executor.run(rx).unwrap();
        println!("rep = {:?}", rep);
        assert_eq!(rep, 1);
    }

    #[test]
    fn client_streaming() {
        #[derive(Debug, Copy, Clone)]
        struct Foo;

        impl ClientStreaming<u64, u64> for Foo {
            type Stream = PinnedStream<u64>;

            type Future = PinnedFuture<u64>;

            fn apply(&mut self, mut req: Self::Stream) -> Self::Future {
                async move {
                    let mut sum = 0_u64;
                    while let Some(n) = await!(req.next()) {
                        sum += n;
                        info!("sum = {}", sum);
                    }
                    sum
                }
                    .boxed()
            }
        }

        init_logging();

        let mut foo = Foo;
        let mut executor = crate::global_executor();
        let sum =
            executor.run(async move { await!(foo.apply(stream::iter(vec![1_u64, 2, 3]).boxed())) });
        assert_eq!(sum, 6);
    }

    #[test]
    fn server_streaming() {
        #[derive(Debug, Copy, Clone)]
        struct Foo;

        impl ServerStreaming<u64, u64> for Foo {
            type Stream = PinnedStream<u64>;

            fn apply(&mut self, req: u64) -> Self::Stream {
                let mut rep = Vec::new();
                for i in 1..=req {
                    rep.push(i)
                }
                stream::iter(rep).boxed()
            }
        }

        init_logging();

        let mut foo = Foo;
        let mut executor = crate::global_executor();
        executor.run(async move {
            let mut rep_stream = foo.apply(3);
            while let Some(n) = await!(rep_stream.next()) {
                info!("n = {}", n);
            }
        });
    }

    #[test]
    fn bidi_streaming() {
        #[derive(Debug, Copy, Clone)]
        struct Foo;

        impl BidirectionalStreaming<u64, u64> for Foo {
            type ReqStream = PinnedStream<u64>;
            type RepStream = PinnedStream<u64>;

            fn apply(&mut self, mut req: Self::ReqStream) -> Self::RepStream {
                let (mut tx, rx) = futures::channel::mpsc::channel(0);
                crate::global_executor()
                    .spawn(async move {
                        while let Some(n) = await!(req.next()) {
                            info!("received request msg: n = {}", n);
                            if await!(tx.send(n * 2)).is_err() {
                                break;
                            }
                        }
                    })
                    .unwrap();
                rx.boxed()
            }
        }

        init_logging();

        let mut foo = Foo;
        let mut executor = crate::global_executor();
        let mut rep_stream = foo.apply(stream::iter(vec![1_u64, 2, 3]).boxed());
        executor.run(async move {
            while let Some(n) = await!(rep_stream.next()) {
                info!("n = {}", n);
            }
        });
    }
}

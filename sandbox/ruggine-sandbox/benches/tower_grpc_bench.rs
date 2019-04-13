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

//! tower-grpc bench tests

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![allow(warnings)]

#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion};

use futures03::{compat::*, future::FutureExt, stream::StreamExt, task::SpawnExt};
use log::*;
use ruggine_async::futures as futures03;

use parking_lot::Mutex;
use std::{env, panic::catch_unwind, sync::Arc};
use tokio::{
    net::{UnixListener, UnixStream},
    runtime::{Runtime, TaskExecutor},
};
use tower::MakeService;
use tower_grpc::codegen::server::grpc;

criterion_group!(benches, unary_bench,);

criterion_main!(benches);

fn unary_bench(c: &mut Criterion) {
    let mut rt = Runtime::new().unwrap();
    let mut executor03 = futures03::executor::ThreadPool::new().unwrap();

    let addr = {
        let mut addr = env::temp_dir();
        addr.push(format!("{}.sock", rusty_ulid::Ulid::generate()));
        addr.into_os_string().into_string().unwrap()
    };
    start_server(rt.executor(), &addr);

    let h2_settings = Default::default();
    let mut make_client = tower_h2::client::Connect::new(
        UnixStreamConnect { addr: addr },
        h2_settings,
        rt.executor(),
    );

    let conn = make_client.make_service(());
    let new_client = async move {
        let conn = await!(conn.compat()).unwrap();
        // the origin header is required by the http2 protocol - without it, the connection is rejected
        let conn_with_origin = tower_request_modifier::Builder::new()
            .set_origin("http://localhost")
            .build(conn)
            .unwrap();
        foo::client::Foo::new(conn_with_origin)
    };
    let client = executor03.run(new_client);
    c.bench_function("unary_bench", move |b| {
        b.iter(|| {
            let mut client = client.clone();
            executor03.run(async {
                let _response = await!(client.unary(grpc::Request::new(foo::Request {})).compat());
            });
        });
    });
}

pub mod foo {
    include!(concat!(
        env!("OUT_DIR"),
        "/oysterpack.ruggine.protos.core.bench.foo_service.rs"
    ));
}

#[derive(Debug, Clone, Default)]
struct FooService;

impl foo::server::Foo for FooService {
    type UnaryFuture =
        Box<dyn futures::Future<Item = grpc::Response<foo::Response>, Error = grpc::Status> + Send>;
    type ClientStreamingFuture =
        Box<dyn futures::Future<Item = grpc::Response<foo::Response>, Error = grpc::Status> + Send>;
    type ServerStreamingStream =
        Box<dyn futures::Stream<Item = foo::Response, Error = grpc::Status> + Send>;
    type ServerStreamingFuture = Box<
        dyn futures::Future<
                Item = grpc::Response<Self::ServerStreamingStream>,
                Error = grpc::Status,
            > + Send,
    >;
    type BidiStreamingStream =
        Box<dyn futures::Stream<Item = foo::Response, Error = grpc::Status> + Send>;
    type BidiStreamingFuture = Box<
        dyn futures::Future<Item = grpc::Response<Self::BidiStreamingStream>, Error = grpc::Status>
            + Send,
    >;

    fn unary(&mut self, request: grpc::Request<foo::Request>) -> Self::UnaryFuture {
        let task = async move { Ok(grpc::Response::new(foo::Response {})) };
        let task = Compat::new(task.boxed());
        Box::new(task)
    }

    fn client_streaming(
        &mut self,
        _request: grpc::Request<grpc::Streaming<foo::Request>>,
    ) -> Self::ClientStreamingFuture {
        unimplemented!()
    }

    fn server_streaming(
        &mut self,
        _request: grpc::Request<foo::Request>,
    ) -> Self::ServerStreamingFuture {
        unimplemented!()
    }

    fn bidi_streaming(
        &mut self,
        _request: grpc::Request<grpc::Streaming<foo::Request>>,
    ) -> Self::BidiStreamingFuture {
        unimplemented!()
    }
}

fn start_server(executor: TaskExecutor, addr: &str) {
    let new_service = foo::server::FooServer::new(FooService::default());

    let h2_settings = Default::default();
    let mut h2 = tower_h2::Server::new(new_service, h2_settings, executor.clone());

    let bind = UnixListener::bind(addr).expect("bind");

    let mut executor03 = executor.clone().compat();
    let server_task = async move {
        let mut incoming = bind.incoming().compat();
        while let Some(sock) = await!(incoming.next()) {
            match sock {
                Ok(sock) => {
                    let serve = h2.serve(sock).compat();
                    let spawn_result = executor03.spawn(async move {
                        if let Err(err) = await!(serve) {
                            error!("h2.serve() error: {:?}", err)
                        }
                    });
                    if let Err(err) = spawn_result {
                        // should never happen
                        error!("failed to spawn task to serve h2 connection: {:?}", err)
                    }
                }
                Err(err) => error!("socket connection error: {:?}", err),
            }
        }
        info!("server is stopped");
    };

    let mut executor03 = executor.compat();
    executor03
        .spawn(server_task)
        .expect("failed to spawn server task");
}

#[derive(Clone)]
struct UnixStreamConnect {
    addr: String,
}

impl tower::Service<()> for UnixStreamConnect {
    type Response = UnixStream;
    type Error = ::std::io::Error;
    type Future = tokio::net::unix::ConnectFuture;

    fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        UnixStream::connect(&self.addr)
    }
}

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

pub mod foo {
    include!(concat!(
        env!("OUT_DIR"),
        "/oysterpack.ruggine.protos.core.foo.rs"
    ));
}

use foo::*;
use log::*;
use ruggine_concurrency::futures as futures03;
use ruggine_concurrency::futures::{
    compat::*,
    future::{FusedFuture, FutureExt},
    stream::StreamExt,
    task::SpawnExt,
};

use futures::{future, prelude::*};
use std::{env, panic::catch_unwind};
use tokio::{
    executor::DefaultExecutor,
    net::{UnixListener, UnixStream},
    runtime::Runtime,
};
use tower_grpc::codegen::server::grpc;

#[derive(Clone, Default)]
struct FooService {
    request_counter: u64,
}

impl foo::server::Foo for FooService {
    type UnaryFuture =
        Box<dyn futures::Future<Item = grpc::Response<Response>, Error = grpc::Status> + Send>;
    type ClientStreamingFuture =
        Box<dyn futures::Future<Item = grpc::Response<Response>, Error = grpc::Status> + Send>;
    type ServerStreamingStream =
        Box<dyn futures::Stream<Item = Response, Error = grpc::Status> + Send>;
    type ServerStreamingFuture = Box<
        dyn futures::Future<
                Item = grpc::Response<Self::ServerStreamingStream>,
                Error = grpc::Status,
            > + Send,
    >;
    type BidiStreamingStream =
        Box<dyn futures::Stream<Item = Response, Error = grpc::Status> + Send>;
    type BidiStreamingFuture = Box<
        dyn futures::Future<Item = grpc::Response<Self::BidiStreamingStream>, Error = grpc::Status>
            + Send,
    >;

    fn unary(&mut self, request: grpc::Request<Request>) -> Self::UnaryFuture {
        unimplemented!()
    }

    fn client_streaming(
        &mut self,
        request: grpc::Request<grpc::Streaming<Request>>,
    ) -> Self::ClientStreamingFuture {
        unimplemented!()
    }

    fn server_streaming(&mut self, request: grpc::Request<Request>) -> Self::ServerStreamingFuture {
        info!("server_streaming(): request: {:?}", request);
        unimplemented!()
    }

    fn bidi_streaming(
        &mut self,
        request: grpc::Request<grpc::Streaming<Request>>,
    ) -> Self::BidiStreamingFuture {
        unimplemented!()
    }
}

fn start_server(rt: &mut Runtime, addr: &str)  {
    let new_service = foo::server::FooServer::new(FooService::default());

    let h2_settings = Default::default();
    let mut h2 = tower_h2::Server::new(new_service, h2_settings, rt.executor());

    let bind = UnixListener::bind(addr).expect("bind");

    let server_task = async move {
        let mut incoming = bind.incoming().compat();
        while let Some(sock) = await!(incoming.next()) {
            let mut executor03 = rt.executor().compat();
            match sock {
                Ok(sock) => {
                    let serve = h2.serve(sock).compat();
                    let spawn_result = executor03.spawn(
                        async move {
                            if let Err(err) = await!(serve) {
                                error!("h2.serve() error: {:?}", err)
                            }
                        },
                    );
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

    let mut executor03 = rt.executor().compat();
    executor03.spawn(server_task);

}

#[derive(Clone)]
struct Dst {
    addr: String,
}

impl tower::Service<()> for Dst {
    type Response = UnixStream;
    type Error = ::std::io::Error;
    type Future = tokio::net::unix::ConnectFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        UnixStream::connect(&self.addr)
    }
}

#[test]
fn grpc_poc() {
    let _ = catch_unwind(env_logger::init);
    let mut rt = Runtime::new().unwrap();

    let addr = {
        let mut addr = env::temp_dir();
        addr.push(format!("{}.sock", rusty_ulid::Ulid::generate()));
        addr.into_os_string().into_string().unwrap()
    };
    start_server(&mut rt, &addr);
}

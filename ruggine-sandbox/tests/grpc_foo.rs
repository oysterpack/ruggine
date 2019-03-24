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

use futures03::{compat::*, future::FutureExt, stream::{
    StreamExt
}, task::{
    SpawnExt
}, sink::SinkExt};
use log::*;
use ruggine_concurrency::futures as futures03;

use parking_lot::Mutex;
use std::{env, panic::catch_unwind, sync::Arc};
use tokio::{
    net::{UnixListener, UnixStream},
    runtime::{Runtime, TaskExecutor},
};
use tower::MakeService;
use tower_grpc::codegen::server::grpc;
use tower::util::ServiceExt;

#[derive(Clone)]
struct FooService {
    value: Arc<Mutex<u64>>,
    executor: futures03::executor::ThreadPool
}

impl Default for FooService{
    fn default() -> Self {
        Self {
            value: Arc::new(Mutex::new(0)),
            executor: futures03::executor::ThreadPool::new().unwrap()
        }
    }
}

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
        info!("unary server request: {:?}", request);
        let value = {
            let mut value = self.value.lock();
            *value += request.get_ref().value;
            *value
        };
        let task = async move {
            let response = grpc::Response::new(foo::Response { value });
            Ok(response)
        };
        let task = Compat::new(task.boxed());
        Box::new(task)
    }

    fn client_streaming(
        &mut self,
        request: grpc::Request<grpc::Streaming<foo::Request>>,
    ) -> Self::ClientStreamingFuture {
        let value = self.value.clone();
        let task = async move {
            let mut request = request.into_inner().compat();
            while let Some(Ok(req_msg)) = await!(request.next()) {
                let mut value = value.lock();
                *value += req_msg.value;
            }
            let value = {
                let value = value.lock();
                *value
            };
            let response = grpc::Response::new(foo::Response { value });
            Ok(response)
        };
        let task = Compat::new(task.boxed());
        Box::new(task)
    }

    fn server_streaming(
        &mut self,
        request: grpc::Request<foo::Request>,
    ) -> Self::ServerStreamingFuture {
        {
            let mut value = self.value.lock();
            *value += request.get_ref().value;
        }
        let (mut tx, rx) = futures03::channel::mpsc::channel(0);
        let server_streaming_task = async move {
            let value = request.get_ref().value;
            for i in 1..=value {
                let response = foo::Response { value: i };
                let response = Result::<_,grpc::Status>::Ok(response);
                if let Err(err) = await!(tx.send(response)) {
                    error!("client has disconnected: {}", err);
                    break;
                }
            }
        };
        self.executor.spawn(server_streaming_task).expect("Failed to spawn server streaming task");

        let server_streaming_future = async move {
            let rx = Compat::new(rx);
            // the compiler needs the type here explicitly specified
            let rx: Box<dyn futures::Stream<Item = foo::Response, Error = grpc::Status> + Send> = Box::new(rx);
            Result::<_,grpc::Status>::Ok(grpc::Response::new(rx))
        };
        Box::new(Compat::new(server_streaming_future.boxed()))

        // using futures 0.1 channels
//        let (mut tx, rx) = ::futures::sync::mpsc::channel(0);
//        let server_streaming_response = async move {
//            use tokio::prelude::Sink;
//            let mut tx = tx;
//            let value = request.get_ref().value;
//            for i in 0..value {
//                let response = foo::Response { value: i };
//                match await!(tx.send(response).compat()) {
//                    Ok(sink) => {
//                        tx = sink
//                    },
//                    Err(err) => {
//                        error!("client has disconnected: {}", err);
//                        break;
//                    }
//                }
//
//            }
//        };
//        self.executor.spawn(server_streaming_response).expect("Failed to spawn server streaming task");
//        use ::futures::stream::Stream;
//        let rx = rx.map_err(|err| grpc::Status::new(grpc::Code::Aborted,"channel disconnected"));
//        // the compiler needs the type here explicitly specified
//        let rx: Box<dyn futures::Stream<Item = foo::Response, Error = grpc::Status> + Send> = Box::new(rx);
//        let grpc_response = grpc::Response::new(rx);
//        Box::new(::futures::future::ok(grpc_response))
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

#[test]
fn grpc_poc() {
    let _ = catch_unwind(env_logger::init);
    let rt = Runtime::new().unwrap();
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
    let client_task = async move {
        let conn = await!(conn.compat()).unwrap();
        // the origin header is required by the http2 protocol - without it, the connection is rejected
        let conn_with_origin = tower_add_origin::Builder::new()
            .uri(format!("https://{}", rusty_ulid::Ulid::generate()))
            .build(conn)
            .unwrap();
        foo::client::Foo::new(conn_with_origin)
    };
    let client = executor03.run(client_task);

    // unary
    {
        let mut client = client.clone();
        executor03.run(async move {
            for _ in 0..10_u8 {
                let request = foo::Request { value: 1, sleep: 0 };
                let response = await!(client.unary(grpc::Request::new(request)).compat());
                info!("unary response = {:?}", response);
            }
        });
    }

    // client streaming
    {
        let mut client = client.clone();
        let mut executor03_clone = executor03.clone();
        executor03.run(async move {
            let (mut tx, rx) = futures03::channel::mpsc::channel(0);

            executor03_clone.spawn(async move {
                for _ in 0..10_u8 {
                    let request = foo::Request { value: 1, sleep: 0 };
                    await!(tx.send(Ok(request))).expect("Failed to send request message on client stream");
                }
            }).expect("failed to spawn client streaming task");

            let rx = Compat::new(rx);
            let response = await!(client.client_streaming(grpc::Request::new(rx)).compat());
            info!("client streaming response = {:?}", response);
        });
    }

    // server streaming
    {
        let mut client = client.clone();
        let mut executor03_clone = executor03.clone();
        executor03.run(async move {
            let request = foo::Request { value: 5, sleep: 0 };
            let grpc_response = await!(client.server_streaming(grpc::Request::new(request)).compat()).unwrap();
            let mut response_stream = grpc_response.into_inner().compat();
            while let Some(msg) = await!(response_stream.next()) {
                info!("server streaming response msg: {:?}", msg);
            }
        });
    }
}

#[test]
fn grpc_poc_using_executor01as03() {
    let _ = catch_unwind(env_logger::init);
    let rt = Runtime::new().unwrap();
    let mut executor01as03 = rt.executor().compat();
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
    let client_task = async move {
        let conn = await!(conn.compat()).unwrap();
        // the origin header is required by the http2 protocol - without it, the connection is rejected
        let conn_with_origin = tower_add_origin::Builder::new()
            .uri(format!("https://{}", rusty_ulid::Ulid::generate()))
            .build(conn)
            .unwrap();
        foo::client::Foo::new(conn_with_origin)
    };
    let client = executor03.run(client_task);

    // unary
    {
        let (mut tx,mut rx) = futures03::channel::mpsc::channel(0);

        let mut client = client.clone();
        executor01as03.spawn(async move {
            for _ in 0..10_u8 {
                let request = foo::Request { value: 1, sleep: 0 };
                let response = await!(client.unary(grpc::Request::new(request)).compat());
                await!(tx.send(response)).unwrap();
            }
        }).expect("failed to spawn client unary task");

        executor03.run(async move {
           while let Some(response) = await!(rx.next()) {
               info!("grpc_poc_using_executor01as03(): unary response = {:?}", response);
           }
        });
    }
}

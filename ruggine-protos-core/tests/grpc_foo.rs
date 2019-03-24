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

pub mod foo {
    include!(concat!(
        env!("OUT_DIR"),
        "/oysterpack.ruggine.protos.core.foo.rs"
    ));
}

use foo::*;
use futures::{future::FutureResult, prelude::*, stream::Stream};
use log::*;
use std::{env, panic::catch_unwind};
use tokio::{
    executor::DefaultExecutor,
    net::{UnixListener, UnixStream},
    prelude::*,
    runtime::Runtime,
};
use tower::MakeService;
use tower_grpc::codegen::server::grpc;

#[derive(Clone, Default)]
struct FooService {
    request_counter: u64,
}

impl foo::server::Foo for FooService {
    type UnaryFuture = FutureResult<grpc::Response<Response>, grpc::Status>;
    type ClientStreamingFuture =
        Box<dyn futures::Future<Item = grpc::Response<Response>, Error = grpc::Status> + Send>;
    type ServerStreamingStream =
        Box<dyn futures::Stream<Item = Response, Error = grpc::Status> + Send>;
    type ServerStreamingFuture =
        FutureResult<grpc::Response<Self::ServerStreamingStream>, grpc::Status>;
    type BidiStreamingStream =
        Box<dyn futures::Stream<Item = Response, Error = grpc::Status> + Send>;
    type BidiStreamingFuture =
        FutureResult<grpc::Response<Self::BidiStreamingStream>, grpc::Status>;

    fn unary(&mut self, request: grpc::Request<Request>) -> Self::UnaryFuture {
        info!("unary(): request: {:?}", request);
        self.request_counter += 1;
        let response = grpc::Response::new(Response {
            count: self.request_counter,
        });
        future::ok(response)
    }

    fn client_streaming(
        &mut self,
        request: grpc::Request<grpc::Streaming<Request>>,
    ) -> Self::ClientStreamingFuture {
        self.request_counter += 1;
        let count = self.request_counter;
        let response = request
            .into_inner()
            .for_each(|req| {
                info!("client_streaming() request: {:?}", req);
                future::ok(())
            })
            .map(move |_| {
                let response = grpc::Response::new(Response { count });
                response
            });
        Box::new(response)
    }

    fn server_streaming(&mut self, request: grpc::Request<Request>) -> Self::ServerStreamingFuture {
        info!("server_streaming(): request: {:?}", request);
        self.request_counter += 1;
        let (mut tx, rx) = tokio::sync::mpsc::unbounded_channel();
        for count in 1..=10 {
            tx.try_send(Response { count })
                .expect("failed to send resuest");
        }
        future::ok(grpc::Response::new(Box::new(rx.map_err(|err| {
            grpc::Status::new(grpc::Code::Aborted, err.to_string())
        }))))
    }

    fn bidi_streaming(
        &mut self,
        request: grpc::Request<grpc::Streaming<Request>>,
    ) -> Self::BidiStreamingFuture {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Response>();

        let task = request
            .into_inner()
            .fold(tx, |tx, req| {
                info!("bidi streaming request: {:?}", req);
                tx.send(Response { count: req.id })
                    .map_err(|err| grpc::Status::new(grpc::Code::Aborted, err.to_string()))
            })
            .map(|_| ())
            .map_err(|_| ());
        tokio::spawn(task);

        let rx = rx.map_err(|err| grpc::Status::new(grpc::Code::Aborted, err.to_string()));
        future::ok::<_, grpc::Status>(grpc::Response::new(Box::new(rx)))
    }
}

fn start_server(rt: &mut Runtime, addr: &str) -> tokio::sync::oneshot::Sender<()> {
    let new_service = foo::server::FooServer::new(FooService::default());

    let h2_settings = Default::default();
    let mut h2 = tower_h2::Server::new(new_service, h2_settings, rt.executor());

    let bind = UnixListener::bind(addr).expect("bind");

    let serve = bind
        .incoming()
        .for_each(move |sock| {
            let serve = h2.serve(sock);
            tokio::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));
            Ok(())
        })
        .map_err(|e| error!("accept error: {}", e));

    // use the channel to signal the server to shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let serve = serve.select2(rx).then(|_| {
        info!("server shutdown has been signalled");
        future::ok::<_, ()>(())
    });
    rt.spawn(serve);
    tx
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
    let tx = start_server(&mut rt, &addr);

    let h2_settings = Default::default();
    let mut make_client =
        tower_h2::client::Connect::new(Dst { addr: addr }, h2_settings, rt.executor());

    let client_task = make_client
        .make_service(())
        .map(move |conn| {
            // the origin header is required by the http2 protocol - without it, the connection is rejected
            let conn = tower_add_origin::Builder::new()
                .uri(format!("https://{}", rusty_ulid::Ulid::generate()))
                .build(conn)
                .unwrap();
            foo::client::Foo::new(conn)
        })
        .and_then(|mut client| {
            let request = foo::Request {
                id: 1,
                sleep: 0,
                futures_version: foo::request::Futures::Three as i32,
            };
            let client_clone = client.clone();
            client
                .unary(grpc::Request::new(request))
                .map(|response| (response, client_clone))
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|(response, mut client)| {
            info!("unary response = {:?}", response);
            let (mut tx, rx) = tokio::sync::mpsc::unbounded_channel();
            for i in 0..10 {
                let request = foo::Request {
                    id: i as u64,
                    sleep: 0,
                    futures_version: foo::request::Futures::Three as i32,
                };
                tx.try_send(request).expect("failed to send resuest");
            }
            let client_clone = client.clone();
            client
                .client_streaming(grpc::Request::new(
                    rx.map_err(|err| grpc::Status::new(grpc::Code::Aborted, err.to_string())),
                ))
                .map(|response| (response, client_clone))
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|(response, mut client)| {
            info!("client streaming response = {:?}", response);
            let request = foo::Request {
                id: 1,
                sleep: 0,
                futures_version: foo::request::Futures::Three as i32,
            };
            let client_clone = client.clone();
            client
                .server_streaming(grpc::Request::new(request))
                .map(|response| (response, client_clone))
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|(response, client)| {
            info!("server streaming response = {:?}", response);
            response
                .into_inner()
                .for_each(|response| {
                    info!("streaming response msg: {:?}", response);
                    future::ok(())
                })
                .then(|_| future::ok(client))
        })
        .and_then(|mut client| {
            let (mut tx, rx) = tokio::sync::mpsc::unbounded_channel();
            for i in 0..10 {
                let request = foo::Request {
                    id: i as u64,
                    sleep: 0,
                    futures_version: foo::request::Futures::Three as i32,
                };
                tx.try_send(request).expect("failed to send resuest");
            }
            client
                .bidi_streaming(grpc::Request::new(
                    rx.map_err(|err| grpc::Status::new(grpc::Code::Aborted, err.to_string())),
                ))
                .and_then(|response| {
                    response.into_inner().for_each(|rep| {
                        info!("received streaming response msg: {:?}", rep);
                        future::ok(())
                    })
                })
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|_| future::ok(()))
        .map_err(|e| {
            info!("ERR = {:?}", e);
        });

    rt.block_on(client_task).unwrap();

    // signal the server to shutdown
    let _ = tx.send(());
}

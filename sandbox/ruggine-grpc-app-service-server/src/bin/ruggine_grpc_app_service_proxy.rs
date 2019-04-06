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

use ruggine_app_service::protos::services::app::v1::{
    client, server, AppInfoRequest, AppInfoResponse,
};
use ruggine_protos_core::app;

use futures03::{compat::*, prelude::*, task::SpawnExt};
use log::*;
use ruggine_async::futures as futures03;
use std::net::{
    SocketAddr, ToSocketAddrs
};
use std::panic::catch_unwind;
use tower::MakeService;

#[derive(Debug, Clone, Copy)]
struct TcpStreamConnect {
    addr: SocketAddr,
}

impl tower::Service<()> for TcpStreamConnect {
    type Response = tokio::net::TcpStream;
    type Error = ::std::io::Error;
    type Future = tokio::net::tcp::ConnectFuture;

    fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        tokio::net::tcp::TcpStream::connect(&self.addr)
    }
}

/// AppService
#[derive(Clone)]
pub struct AppServiceProxy {
    connect: TcpStreamConnect,
}

impl server::AppService for AppServiceProxy {
    type AppInfoFuture = Box<
        dyn futures::Future<
                Item = tower_grpc::Response<AppInfoResponse>,
                Error = tower_grpc::Status,
            > + Send,
    >;

    fn app_info(&mut self, request: tower_grpc::Request<AppInfoRequest>) -> Self::AppInfoFuture {
        let connect = self.connect;
        let response = async move {
            let h2_settings = Default::default();
            let mut make_client = tower_h2::client::Connect::new(
                connect,
                h2_settings,
                ruggine_async::global_executor().compat(),
            );
            let conn = make_client.make_service(());
            let conn = await!(conn.compat()).unwrap();
            // the origin header is required by the http2 protocol - without it, the connection is rejected
            let conn_with_origin = tower_add_origin::Builder::new()
                .uri("http://localhost")
                .build(conn)
                .unwrap();
            let mut app_service_client = client::AppService::new(conn_with_origin);
            await!(app_service_client.app_info(request).compat())
        }
            .boxed()
            .compat();

        Box::new(response)
    }
}

fn main() {
    let _ = catch_unwind(env_logger::init);
    let app = app!();
    info!("app is running: {:#?}", app);

    let executor = ruggine_async::global_executor();
    let addr = "localhost:50501".to_socket_addrs().unwrap().next().unwrap();
    let (server, _handle) = grpc_server(addr, executor, 50502);
    let _ = ruggine_async::global_executor().run(server);
}

/// Create grpc server future
/// TODO: track number of connections
/// TODO: limit the number of connections
pub fn grpc_server(
    addr: SocketAddr,
    executor: futures03::executor::ThreadPool,
    port: usize
) -> (
    futures03::future::Abortable<impl futures03::Future<Output = ()>>,
    futures03::future::AbortHandle,
) {
    let new_service = server::AppServiceServer::new(AppServiceProxy {
        connect: TcpStreamConnect { addr },
    });

    let h2_settings = Default::default();
    let mut h2 = tower_h2::Server::new(new_service, h2_settings, executor.clone().compat());

    let addr = format!("0.0.0.0:{}", port);
    let addr = addr.parse().unwrap();
    let bind = tokio::net::tcp::TcpListener::bind(&addr).expect("bind");

    let server_task = {
        let mut executor = executor.clone();
        async move {
            info!("Server is running ...");
            let mut incoming = bind.incoming().compat();
            while let Some(sock) = await!(incoming.next()) {
                match sock {
                    Ok(sock) => {
                        let serve = h2.serve(sock).compat();
                        let spawn_result = executor.spawn(
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
        }
    };

    futures03::future::abortable(server_task)
}

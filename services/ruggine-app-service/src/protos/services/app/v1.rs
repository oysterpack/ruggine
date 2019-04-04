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

//! App grpc service: oysterpack.ruggine.protos.core.services.app.v1.App

use ruggine_async::futures as futures03;
use ruggine_protos_core::protos::messages::app::v1::App;
use futures03::{
    compat::*, prelude::*, task::SpawnExt
};
use log::*;

include!(concat!(
    env!("OUT_DIR"),
    "/oysterpack.ruggine.protos.core.services.app.v1.rs"
));

/// AppService
#[derive(Clone)]
pub struct AppService {
    app: App,
}

impl server::AppService for AppService {
    type AppInfoFuture =
        futures::future::FutureResult<tower_grpc::Response<AppInfoResponse>, tower_grpc::Status>;

    fn app_info(&mut self, _request: tower_grpc::Request<AppInfoRequest>) -> Self::AppInfoFuture {
        let app_info_response = AppInfoResponse {
            app: Some(self.app.clone()),
        };
        let response = tower_grpc::Response::new(app_info_response);
        futures::future::ok(response)
    }
}

/// AppServiceServer constructor
pub fn new_server(app: App) -> server::AppServiceServer<AppService> {
    server::AppServiceServer::new(AppService { app })
}

/// spawns a new server and returns an AbortHandle that can be used to abort the server
pub fn spawn_server(
    app: App,
    mut executor: futures03::executor::ThreadPool,
    port: usize,
) -> futures03::future::AbortHandle {

    let new_service = new_server(app);

    let h2_settings = Default::default();
    let mut h2 = tower_h2::Server::new(new_service, h2_settings, executor.clone().compat());

    let addr = format!("127.0.0.1:{}", port);
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

    let (abortable_server_task, abort_handle) = futures03::future::abortable(server_task);
    executor
        .spawn(async move {
            if let Err(_aborted) = await!(abortable_server_task) {
                info!("Server was aborted");
            }
        })
        .expect("failed to spawn server task");
    abort_handle
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::panic::catch_unwind;
    use tower::MakeService;
    use std::time::Duration;

    struct TcpStreamConnect {
        port: u16
    }

    impl tower::Service<()> for TcpStreamConnect {
        type Response = tokio::net::TcpStream;
        type Error = ::std::io::Error;
        type Future = tokio::net::tcp::ConnectFuture;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, _: ()) -> Self::Future {
            tokio::net::tcp::TcpStream::connect(&([127, 0, 0, 1], self.port).into())
//            let addr = format!("127.0.0.1:{}", self.port).parse().unwrap();
//            tokio::net::tcp::TcpStream::connect(&addr)
        }
    }

    #[test]
    fn app_service_grpc() {
        let _ = catch_unwind(env_logger::init);
        let app = ruggine_protos_core::app!();
        info!("{:#?}", app);

        let port = 50500;
        let server_handle = spawn_server(app,  ruggine_async::global_executor(), port);

        let h2_settings = Default::default();
        let mut make_client = tower_h2::client::Connect::new(
            TcpStreamConnect { port: port as u16 },
            h2_settings,
            ruggine_async::global_executor().compat(),
        );
        let conn = make_client.make_service(());
        let client_task = async move {
            let conn = await!(conn.compat()).unwrap();
            // the origin header is required by the http2 protocol - without it, the connection is rejected
            let conn_with_origin = tower_add_origin::Builder::new()
                .uri(format!("http://localhost:{}", port))
                .build(conn)
                .unwrap();
            super::client::AppService::new(conn_with_origin)
        };

        let mut executor = ruggine_async::global_executor();
        let mut client = executor.run(client_task);

        let response_future = client.app_info(tower_grpc::Request::new(super::AppInfoRequest{}));
        let response = executor.run(async move {
           await!(response_future.compat())
        });
        info!("response: {:#?}", response);

         server_handle.abort();
    }
}

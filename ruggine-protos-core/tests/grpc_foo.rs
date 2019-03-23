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
use tokio::prelude::*;
use tower_grpc::codegen::server::*;

#[derive(Clone)]
struct FooService;

impl foo::server::Foo for FooService {
    type UnaryFuture = futures::FutureResult<grpc::Response<Response>, grpc::Status>;
    type ClientStreamingFuture = futures::FutureResult<grpc::Response<Response>, grpc::Status>;
    type ServerStreamingStream = Box<dyn futures::Stream<Item = Response, Error = grpc::Status>>;
    type ServerStreamingFuture =
        futures::FutureResult<grpc::Response<Self::ServerStreamingStream>, grpc::Status>;
    type BidiStreamingStream = Box<dyn futures::Stream<Item = Response, Error = grpc::Status>>;
    type BidiStreamingFuture = Box<
        dyn futures::Future<Item = grpc::Response<Self::BidiStreamingStream>, Error = grpc::Status>,
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
        unimplemented!()
    }

    fn bidi_streaming(
        &mut self,
        request: grpc::Request<grpc::Streaming<Request>>,
    ) -> Self::BidiStreamingFuture {
        unimplemented!()
    }
}

#[test]
fn unary() {
    // TODO
}

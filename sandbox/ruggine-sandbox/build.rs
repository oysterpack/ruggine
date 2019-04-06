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

fn main() {
    generate_tests_grpc_code();
    generate_benches_grpc_code();
}

fn generate_tests_grpc_code() {
    tower_grpc_build::Config::new()
        .enable_client(true)
        .enable_server(true)
        .build(&["tests/protos/foo.proto"], &["tests/protos"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}

fn generate_benches_grpc_code() {
    tower_grpc_build::Config::new()
        .enable_client(true)
        .enable_server(true)
        .build(&["benches/protos/foo.proto"], &["benches/protos"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}

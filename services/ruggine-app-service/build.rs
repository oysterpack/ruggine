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
    generate_gprc_server();
}

fn generate_gprc_server() {
    let mut config = prost_build::Config::new();
    config
        .extern_path(
            ".oysterpack.ruggine.protos.core.messages.app.v1",
            "::ruggine_protos_core::protos::messages::app::v1",
        )
        .extern_path(
            ".oysterpack.ruggine.protos.core.messages.time.v1",
            "::ruggine_protos_core::protos::messages::time::v1",
        )
        .extern_path(
            ".oysterpack.ruggine.protos.core.messages.ulid.v1",
            "::ruggine_protos_core::protos::messages::ulid::v1",
        );

    tower_grpc_build::Config::from_prost(config)
        .enable_client(true)
        .enable_server(true)
        .build(
            &["services/ruggine-app-service/protos/services/app_v1.proto"],
            &["../.."],
        )
        .unwrap_or_else(|e| panic!("generate_message_protobufs() failed: {}", e));
}

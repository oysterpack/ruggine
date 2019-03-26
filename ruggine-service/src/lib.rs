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

//! OysterPack ruggine concurrency support

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/ruggine_service/0.1.0")]

/// re-exporting futures 0.3 API in order to make it easy to use it side by side with futures 0.1 API
pub use futures;

pub mod service;

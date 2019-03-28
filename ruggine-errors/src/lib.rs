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

//! OysterPack ruggine core

#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/ruggine_errors/0.1.0")]

use failure::*;

/// Signifies an error is never expected to happen.
/// This is meant to be used where the API requires an error type to be specified, but an error is impossible.
#[derive(Debug, Fail)]
#[fail(display = "Never")]
pub struct Never;

// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

mod api;
mod engine;
mod render;

use wasm_bindgen::prelude::*;

/// Solve a JSON request and return a JSON response.
#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    api::solve_json(input)
}

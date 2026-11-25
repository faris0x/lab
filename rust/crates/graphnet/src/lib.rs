// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

mod engine;

use wasm_bindgen::prelude::*;

/// Solve a JSON request and return a JSON response.
#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    engine::solve_json(input)
}

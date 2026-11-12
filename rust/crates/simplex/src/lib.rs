mod api;
mod engine;
mod render;

use wasm_bindgen::prelude::*;

/// Solve a JSON request and return a JSON response.
#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    api::solve_json(input)
}

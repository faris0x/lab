//! Exact Taylor and Maclaurin series with general-term detection.

mod api;
mod detect;
mod expr;
mod parser;
mod render;

use wasm_bindgen::prelude::*;

/// Solve a JSON request and return a JSON response.
#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    api::solve_json(input)
}

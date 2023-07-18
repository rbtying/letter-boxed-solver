use std::io::Write;

use letter_boxed_solver::LetterBoxed;

mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn solve(
    side_1: &str,
    side_2: &str,
    side_3: &str,
    side_4: &str,
    prior_words: &str,
    depth: usize,
) -> String {
    let b = LetterBoxed::load_board(&[side_1, side_2, side_3, side_4]);

    let prior_words = prior_words.split_ascii_whitespace().collect::<Vec<_>>();

    let mut out = vec![];
    for (result, score) in b.solve_with_builtin_list(&prior_words, depth, 25) {
        write!(
            &mut out,
            "{}/{}",
            score,
            side_1.len() + side_2.len() + side_3.len() + side_4.len()
        )
        .unwrap();
        for word in result {
            write!(&mut out, " {}", word).unwrap();
        }
        writeln!(&mut out).unwrap();
        writeln!(&mut out).unwrap();
    }

    String::from_utf8_lossy(&out).to_string()
}

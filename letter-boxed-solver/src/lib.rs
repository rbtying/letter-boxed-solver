use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::sync::OnceLock;

/// A basic solver for the New York Times "Letter Boxed" puzzle.
///
/// The puzzle is set up as a square, where there are letters on each side of
/// the square. Letter transitions within a word must cross from one side of the
/// square to another, and the first letter of the next word must match the last
/// letter of the current word.
///
/// The goal of the game is to use all of the letters with the fewest number of
/// words.
///
/// For example:
///
///   E L Z
/// I       C
/// V       T
/// A       H
///   R Y U
///
/// has a valid solution of "VEHICULAR" followed by "RITZILY".
///
#[derive(Debug)]
pub struct LetterBoxed {
    /// Letters which are not permitted to be adjacent to one another
    nonadjacent: HashSet<(char, char)>,
    /// For convenience, all of the letters that are in the puzzle.
    letters: BTreeSet<char>,
}

impl LetterBoxed {
    /// Load the board. For simplicity, just take a slice representing each side.
    /// Order does not matter.
    pub fn load_board(sides: &[&str]) -> LetterBoxed {
        let mut nonadjacent = HashSet::new();

        for side in sides {
            for c in side.chars() {
                for cc in side.chars() {
                    nonadjacent.insert((c, cc));
                }
            }
        }
        let letters = sides
            .iter()
            .flat_map(|s| s.chars())
            .collect::<BTreeSet<char>>();

        LetterBoxed {
            nonadjacent,
            letters,
        }
    }

    /// Validate that a given solution is correct on this board.
    pub fn validate(&self, solution: &[&str]) -> bool {
        for window in solution.windows(2) {
            if window[0].chars().last() != window[1].chars().next() {
                return false;
            }
        }
        for word in solution {
            let mut iter = word.chars();
            let mut current = iter.next();

            for next in iter {
                if let Some(c) = current {
                    if self.nonadjacent.contains(&(c, next)) {
                        return false;
                    }
                    current = Some(next);
                } else {
                    return false;
                }
            }
        }
        true
    }

    /// Solve using a built-in hardcoded word list, where all solutions will not
    /// exceed `max_depth` in length.
    ///
    /// The solver prefers shorter solutions to longer solutions, and will
    /// return up to `max_results` solutions.
    ///
    /// `prior_words` are words (all-caps) which have already been played. This
    /// will crash if an element in `prior_words` is not in the builtin word list.
    pub fn solve_with_builtin_list(
        &self,
        prior_words: &[&str],
        max_depth: usize,
        max_results: usize,
    ) -> Vec<(Vec<&'static str>, usize)> {
        static WORDS_LIST: OnceLock<Vec<&'static str>> = OnceLock::new();
        let words = WORDS_LIST.get_or_init(|| WORDS.lines().map(|w| w.trim()).collect::<Vec<_>>());
        let mut prior_words_indices = vec![];
        for w in prior_words {
            let idx = words.iter().position(|ww| ww == w).unwrap();
            prior_words_indices.push(idx);
        }
        self.solve(words, &prior_words_indices, max_depth, max_results)
    }

    /// Solve using a provided word list, where all solutions will not
    /// exceed `max_depth` in length.
    ///
    /// `prior_words_indices` should correspond to any words that have already
    /// been played, represented as indices into `words`.
    pub fn solve<'word>(
        &self,
        words: &[&'word str],
        prior_words_indices: &[usize],
        max_depth: usize,
        max_results: usize,
    ) -> Vec<(Vec<&'word str>, usize)> {
        let mut results = vec![];

        // The graph maps from a start-letter to an end-letter, with each
        // possible word that bridges them according to the board as a potential
        // route.
        //
        // In the above example board (replicated here)
        //
        //   E L Z
        // I       C
        // V       T
        // A       H
        //   R Y U
        //
        // This would include an entry 'V' -> 'R' {..., "VEHICULAR", ...}
        let mut graph: BTreeMap<char, BTreeMap<char, BTreeSet<usize>>> = BTreeMap::new();

        'outer: for (i, w) in words.iter().enumerate() {
            let w = w.trim();
            // Eliminate words that are too short, and those which contain
            // letters not on the board at all
            if w.len() < 3 || w.chars().any(|c| !self.letters.contains(&c)) {
                continue;
            }

            let mut c_iter = w.chars();
            let first_char = c_iter.next().unwrap();
            let mut current_char = first_char;

            // Check that adjacent characters are not in the known-nonadjacent set.
            for c in c_iter {
                let c = c;
                if self.nonadjacent.contains(&(current_char, c)) {
                    continue 'outer;
                }
                current_char = c;
            }

            let options = graph.entry(first_char).or_default();
            options.entry(current_char).or_default().insert(i);
        }

        /// State for the word-search.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct State {
            /// The current letter we are starting from
            cur: char,
            /// All the letters we've visited on this path
            visited: BTreeSet<char>,
            path: Vec<usize>,
        }

        let mut q = VecDeque::new();

        let mut best = (0, vec![]);

        if prior_words_indices.is_empty() {
            // Preload the queue at each possible start location
            for k in graph.keys() {
                let mut visited = BTreeSet::new();
                visited.insert(*k);
                q.push_back(State {
                    cur: *k,
                    visited,
                    path: vec![],
                })
            }
        } else {
            let last_c = words[prior_words_indices[prior_words_indices.len() - 1]]
                .chars()
                .last()
                .unwrap();
            let mut visited = BTreeSet::new();

            for idx in prior_words_indices {
                visited.extend(words[*idx].chars());
            }

            q.push_back(State {
                cur: last_c,
                visited,
                path: prior_words_indices.to_vec(),
            })
        }

        while let Some(state) = q.pop_front() {
            // Keep track of the best-available solution, since we might not
            // find one with the given max_depth.
            if state.visited.len() > best.0
                || (state.visited.len() == best.0 && state.path.len() < best.1.len())
            {
                best = (state.visited.len(), state.path.clone());
            }

            // Check if we're done!
            if state.visited == self.letters {
                results.push((state.path.clone(), self.letters.len()));

                if results.len() >= max_results {
                    break;
                }
            } else if let Some(options) = graph.get(&state.cur) {
                if state.path.len() + 1 > max_depth {
                    continue;
                }
                // Go through all the potential end-letters
                for (next_letter, word_indices) in options {
                    // and all the paths to get there
                    for idx in word_indices {
                        let w = words[*idx];
                        // only consider routes that add a new word to the visited set
                        if w.chars().any(|c| !state.visited.contains(&c)) {
                            let mut v = state.visited.clone();
                            v.extend(w.chars());

                            let mut new_path = state.path.clone();
                            new_path.push(*idx);

                            let new_state = State {
                                cur: *next_letter,
                                visited: v,
                                path: new_path,
                            };

                            q.push_back(new_state);
                        }
                    }
                }
            }
        }

        // if we couldn't find any complete results, add the best one we found to the output.
        if results.is_empty() {
            results.push((best.1, best.0));
        }

        results
            .into_iter()
            .map(|(idxes, c)| (idxes.into_iter().map(|idx| words[idx]).collect(), c))
            .collect()
    }
}

const WORDS: &str = include_str!("words.txt");

#[cfg(test)]
mod tests {
    use super::LetterBoxed;

    #[test]
    fn test_1() {
        let b = LetterBoxed::load_board(&["OAL", "NUK", "CET", "RPI"]);
        let results = b.solve_with_builtin_list(&[], 3, 25);
        assert!(!results.is_empty());
        for r in results {
            assert!(b.validate(&r.0));
        }
    }

    #[test]
    fn test_2() {
        let b = LetterBoxed::load_board(&["ELZ", "IVA", "RYU", "CTH"]);
        let results = b.solve_with_builtin_list(&[], 3, 25);
        assert!(!results.is_empty());
        for r in results {
            assert!(b.validate(&r.0));
        }
    }

    #[test]
    fn test_3() {
        let b = LetterBoxed::load_board(&["RTF", "USY", "HIA", "OEB"]);
        let results = b.solve_with_builtin_list(&[], 2, 25);
        assert!(!results.is_empty());
        for r in results {
            assert!(b.validate(&r.0));
        }
    }

    #[test]
    fn test_4() {
        let b = LetterBoxed::load_board(&["RTF", "USY", "HIA", "OEB"]);
        let results = b.solve_with_builtin_list(&["STATUTORY"], 2, 25);
        assert!(!results.is_empty());
        for r in results {
            assert!(b.validate(&r.0));
        }
    }
}

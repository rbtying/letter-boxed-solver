use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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
    /// Note that the solver does not guarantee any solutions of `max_depth`
    /// length, as it prefers shorter solutions to longer ones.
    pub fn solve_with_builtin_list(&self, max_depth: usize) -> Vec<(Vec<&'static str>, usize)> {
        static WORDS_LIST: OnceLock<Vec<&'static str>> = OnceLock::new();
        let words = WORDS_LIST.get_or_init(|| WORDS.lines().map(|w| w.trim()).collect::<Vec<_>>());
        self.solve(words, max_depth)
    }

    /// Solve using a provided word list, where all solutions will not
    /// exceed `max_depth` in length.
    ///
    /// Note that the solver does not guarantee any solutions of `max_depth`
    /// length, as it prefers shorter solutions to longer ones.
    pub fn solve<'word>(
        &self,
        words: &[&'word str],
        max_depth: usize,
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
        let mut graph: BTreeMap<char, BTreeMap<char, BTreeSet<&str>>> = BTreeMap::new();

        'outer: for w in words {
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
            options.entry(current_char).or_default().insert(w);
        }

        /// State for the word-search.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct State {
            /// The current letter we are starting from
            cur: char,
            /// All the letters we've visited on this path
            visited: BTreeSet<char>,
        }

        // We keep a backtrack map which points from a given state to its
        // parent state, as well as the word used to get there and (for
        // convenient) the depth of the path to get to that state.
        let mut backtrack: HashMap<State, (State, &str, usize)> = HashMap::new();

        let mut stk = vec![];

        let mut best = (0, vec![]);

        // Preload the stack at each possible start location
        for k in graph.keys() {
            let mut visited = BTreeSet::new();
            visited.insert(*k);
            stk.push(State { cur: *k, visited })
        }

        while let Some(state) = stk.pop() {
            let mut path = vec![];
            let mut ptr = &state;

            // This runs in log time, so probably not worth optimizing quite
            // yet. We could cache it in `backtrack`, similar to the depth.
            while let Some(parent) = backtrack.get(ptr) {
                path.push(parent.1);
                ptr = &parent.0;
            }
            path.reverse();

            // Bail out if the length is too long.
            if path.len() + 1 > max_depth {
                continue;
            }

            // Keep track of the best-available solution, since we might not
            // find one with the given max_depth.
            if state.visited.len() > best.0
                || (state.visited.len() == best.0 && path.len() < best.1.len())
            {
                best = (state.visited.len(), path.clone());
            }

            // Check if we're done!
            if state.visited == self.letters {
                results.push((path.clone(), self.letters.len()));
            } else if let Some(options) = graph.get(&state.cur) {
                // Go through all the potential end-letters
                for (next_letter, words) in options {
                    // and all the paths to get there
                    for w in words {
                        // only consider routes that add a new word to the visited set
                        if w.chars().any(|c| !state.visited.contains(&c)) {
                            let mut v = state.visited.clone();
                            v.extend(w.chars());

                            let new_state = State {
                                cur: *next_letter,
                                visited: v,
                            };

                            // Update this state in the backtrack table (and add
                            // it to the stack) if:
                            //
                            // 1. we've never been there before, or,
                            // 2. we've been there before, but using a longer
                            //    path than the one we took to get here.
                            if backtrack.get(&new_state).map(|s| s.2).unwrap_or(usize::MAX)
                                > path.len()
                            {
                                backtrack.insert(new_state.clone(), (state.clone(), w, path.len()));
                                stk.push(new_state);
                            }
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
    }
}

const WORDS: &str = include_str!("words.txt");

#[cfg(test)]
mod tests {
    use super::LetterBoxed;

    #[test]
    fn test_1() {
        let b = LetterBoxed::load_board(&["OAL", "NUK", "CET", "RPI"]);
        let results = b.solve_with_builtin_list(3);
        assert!(!results.is_empty());
        for r in results {
            assert!(b.validate(&r.0));
        }
    }

    #[test]
    fn test_2() {
        let b = LetterBoxed::load_board(&["ELZ", "IVA", "RYU", "CTH"]);
        let results = b.solve_with_builtin_list(4);
        assert!(!results.is_empty());
        for r in results {
            assert!(b.validate(&r.0));
        }
    }
}

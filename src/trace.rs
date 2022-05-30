//! Contains the `Trace` struct.

use std::fmt::Display;

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Trace {
    trace: Vec<u32>,
}

impl From<Vec<u32>> for Trace {
    fn from(trace: Vec<u32>) -> Self {
        Self { trace }
    }
}

impl Trace {
    // Calculate the stack distances per-operation.
    //
    // Returns a vector where the ith entry represents the stack distance at that point.
    fn stack_distance(&self) -> Vec<Option<usize>> {
        let mut out = vec![Some(0); self.trace.len()];

        let mut stack = Vec::new();

        for (i, curr) in self.trace.iter().enumerate() {
            let position = stack.iter().position(|n| n == &curr);
            out[i] = position.map(|n| stack.len() - n - 1); // the stack is right-to-left
            if let Some(position) = position {
                stack.remove(position);
            }
            stack.push(curr);
        }

        out
    }

    /// Calculate the stack distance histogram.
    ///
    /// Returns a vector of frequencies of stack distances, plus the count of intinities.
    pub fn stack_distance_histogram(&self) -> (Vec<usize>, usize) {
        let distances = self.stack_distance();
        let max = distances.iter().flatten().max();

        let mut freqs = max.map_or_else(Vec::new, |max| vec![0; max + 1]);

        let mut infinities = 0;

        for i in distances {
            #[allow(clippy::option_if_let_else)]
            if let Some(i) = i {
                freqs[i] += 1;
            } else {
                infinities += 1;
            }
        }

        (freqs, infinities)
    }

    /// Calculate the frequency historgram.
    ///
    /// Returns a vector of frequencies of accesses.
    pub fn frequency_histogram(&self) -> Vec<usize> {
        let mut freqs = vec![0; self.trace.iter().max().map_or(0, |n| n + 1) as usize];

        for i in &self.trace {
            freqs[*i as usize] += 1;
        }

        freqs
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.trace.iter().max().map_or(true, |&n| n < 26) {
            for i in &self.trace {
                write!(
                    f,
                    "{}",
                    char::from_u32(i + 'A' as u32).expect("all elements of list are valid chars")
                )?;
            }
        } else {
            for i in &self.trace {
                write!(f, "{} ", i)?;
            }
        }
        Ok(())
    }
}

pub struct TraceIter {
    next: Option<Vec<u32>>,
}

impl TraceIter {
    pub fn new(trace_size: usize) -> Self {
        Self {
            next: Some(vec![0; trace_size]),
        }
    }
}

impl Iterator for TraceIter {
    type Item = Trace;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next.clone();

        if let Some(next) = &mut self.next {
            let current_symbols: Vec<_> = next.iter().unique().collect();

            for (i, n) in next.clone().iter().enumerate().rev() {
                // unique is stable, so we can just get the next element
                let current_symbol_index = current_symbols
                    .iter()
                    .position(|&x| x == n)
                    .expect("The current symbol is in the list of current symbols.");

                if current_symbol_index == current_symbols.len() - 1 {
                    // at the end of the list of symbols
                    // two options: either we're already on a unique symbol for this value, or we
                    // need to go to a unique symbol.

                    if next
                        .iter()
                        .position(|x| x == n)
                        .expect("The current symbol is in the trace.")
                        != i
                    {
                        // first occurance is at another access, we can increment this one to get
                        // a new symbol
                        next[i] += 1;
                        for j in next.iter_mut().skip(i + 1) {
                            // reset everything else to 0, since we need to redo everything after
                            // this
                            *j = 0;
                        }
                        break;
                    }
                } else if i == 0 {
                    // here we get to the beginning of the trace, which should always just be 0 (up
                    // to renaming), so we're done with iteration
                    self.next = None;
                    break; // needed for borrow-checker, obviously if i=0 the loop ends here anyway
                } else if n <= &next[i - 1] {
                    // we have this extra constraint to prevent the following "runaway" case:
                    //
                    // AA
                    // AB
                    // BA
                    // BB
                    // BC
                    // etc.
                    //
                    // There's no reason for the ith access to be bigger than one more than the
                    // i-1st access, given we want renaming symmetry. This check guarantees that.
                    next[i] = *current_symbols[current_symbol_index + 1];
                    for j in next.iter_mut().skip(i + 1) {
                        // reset everything else to 0, since we need to redo everything after
                        // this
                        *j = 0;
                    }
                    break;
                }
            }
        }

        ret.map(Trace::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    mod stack_distance {
        use super::*;

        macro_rules! stack_distance_test {
            ($name:ident: $($in:expr),* => $($out:expr),*) => {
                #[test]
                fn $name() {
                    assert_eq!(Trace::from(vec![$($in),*]).stack_distance(), vec![$($out),*])
                }
            };
        }

        stack_distance_test!(basic: 1, 2, 3 => None, None, None);
        stack_distance_test!(repeated: 1, 1, 1 => None, Some(0), Some(0));
        stack_distance_test!(one_two: 1, 2, 1, 1, 1 => None, None, Some(1), Some(0), Some(0));
        stack_distance_test!(one_repeated: 1, 2, 3, 1 => None, None, None, Some(2));
        stack_distance_test!(empty: => );
    }

    mod stack_distance_histograms {
        use super::*;

        macro_rules! stack_distance_histogram_test {
            ($name:ident: $($in:expr),* => $($out:expr),*; $infinities:expr) => {
                #[test]
                fn $name() {
                    let (freqs, infinities) = Trace::from(vec![$($in),*]).stack_distance_histogram();
                    assert_eq!(infinities, $infinities);
                    assert_eq!(freqs, vec![$($out),*]);
                }
            };
        }

        stack_distance_histogram_test!(basic: 1, 2, 3 => ; 3);
        stack_distance_histogram_test!(repeated: 1, 1, 1 => 2; 1);
        stack_distance_histogram_test!(one_two: 1, 2, 1, 1, 1 => 2, 1; 2);
        stack_distance_histogram_test!(one_repeated: 1, 2, 3, 1 => 0, 0, 1; 3);
        stack_distance_histogram_test!(empty: => ; 0);
    }

    mod frequency {
        use super::*;

        macro_rules! frequency_test {
            ($name:ident: $($in:expr),* => $($out:expr),*) => {
                #[test]
                fn $name() {
                    assert_eq!(Trace::from(vec![$($in),*]).frequency_histogram(), vec![$($out),*])
                }
            };
        }

        frequency_test!(basic: 1, 2, 3 => 0, 1, 1, 1);
        frequency_test!(repeated: 1, 1, 1 => 0, 3);
        frequency_test!(one_two: 1, 2, 1, 1, 1 => 0, 4, 1);
        frequency_test!(one_repeated: 1, 2, 3, 1 => 0, 2, 1, 1);
        frequency_test!(empty: => );
    }

    #[test]
    fn trace_iter_works_three() {
        assert_eq!(
            TraceIter::new(3).collect::<HashSet<_>>(),
            HashSet::from([
                Trace::from(vec![0, 0, 0]),
                Trace::from(vec![0, 0, 1]),
                Trace::from(vec![0, 1, 0]),
                Trace::from(vec![0, 1, 1]),
                Trace::from(vec![0, 1, 2]),
            ])
        );
    }

    #[test]
    fn trace_iter_works_four() {
        assert_eq!(
            TraceIter::new(4).collect::<HashSet<_>>(),
            HashSet::from([
                Trace::from(vec![0, 0, 0, 0]),
                Trace::from(vec![0, 0, 0, 1]),
                Trace::from(vec![0, 0, 1, 0]),
                Trace::from(vec![0, 0, 1, 1]),
                Trace::from(vec![0, 0, 1, 2]),
                Trace::from(vec![0, 1, 0, 0]),
                Trace::from(vec![0, 1, 0, 1]),
                Trace::from(vec![0, 1, 0, 2]),
                Trace::from(vec![0, 1, 1, 0]),
                Trace::from(vec![0, 1, 1, 1]),
                Trace::from(vec![0, 1, 1, 2]),
                Trace::from(vec![0, 1, 2, 0]),
                Trace::from(vec![0, 1, 2, 1]),
                Trace::from(vec![0, 1, 2, 2]),
                Trace::from(vec![0, 1, 2, 3]),
            ])
        );
    }
}

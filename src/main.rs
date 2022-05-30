mod trace;

use trace::{Trace, TraceIter};

fn compare(t: Trace) {
    let (stack_distances, infinities) = t.stack_distance_histogram();
    let frequencies = t.frequency_histogram();

    // an infinity means a new variable, so it should be equal to the number of non-zero elements
    // of frequencies
    assert_eq!(infinities, frequencies.iter().filter(|&&n| n != 0).count());
}

fn main() {
    const TRACE_SIZE: usize = 4;

    for trace in TraceIter::new(TRACE_SIZE) {
        // println!("{}", trace);
        compare(trace);
    }
}

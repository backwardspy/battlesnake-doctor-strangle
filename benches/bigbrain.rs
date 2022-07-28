#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use battlesnake_doctor_strangle::strategies::strangle::{
        bench::make_game,
        brain::{bigbrain, BigbrainOptions},
    };
    use test::Bencher;

    #[bench]
    fn bench_bigbrain(b: &mut Bencher) {
        const DEPTH: u64 = 2;
        const TIME_LIMIT: Duration = Duration::from_secs(1);

        let game = make_game(4, 19, 19);
        b.iter(|| {
            bigbrain(
                &game,
                0,
                0,
                &HashMap::new(),
                &mut HashMap::new(),
                Instant::now(),
                BigbrainOptions {
                    max_depth:  DEPTH,
                    time_limit: TIME_LIMIT,
                    trace_sim:  false,
                },
            )
        });
    }
}

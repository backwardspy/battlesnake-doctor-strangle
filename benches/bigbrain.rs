#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use battlesnake_doctor_strangle::strategies::strangle::{
        bench::make_game,
        brain::bigbrain,
    };
    use test::Bencher;

    #[bench]
    fn bench_bigbrain(b: &mut Bencher) {
        const DEPTH: u64 = 2;

        let game = make_game(4, 19, 19);
        b.iter(|| bigbrain(&game, 0, 0, DEPTH, &HashMap::new(), false));
    }
}

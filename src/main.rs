mod profiler;

use kvdb::{DBOp, DBTransaction};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use std::time::{Duration, Instant};

fn insert_op<R: Rng>(rng: &mut R) -> DBOp {
    DBOp::Insert {
        col: 0,
        key: rng.gen::<[u8; 32]>().into(),
        value: rng.gen::<[u8; 32]>().to_vec(),
    }
}

fn main() {
    let mut rng = Pcg64::seed_from_u64(3);
    let db = kvdb_rocksdb::Database::open(&Default::default(), "./__tmp").unwrap();

    let mut round: usize = 0;
    let mut first_slow_round: Option<usize> = None;
    let mut profiler = profiler::Profiler::new(250);

    loop {
        // In each round, write 1000 random key-value pairs first.
        let ops = (0..1000).map(|_| insert_op(&mut rng)).collect();
        db.write(DBTransaction { ops }).unwrap();

        // Time 1000 read operations.
        let start_time = Instant::now();
        for _ in 0..1000 {
            db.get(0, &rng.gen::<[u8; 32]>()).unwrap();
        }
        println!("Round {}, Time elapsed {:?}", round, start_time.elapsed());

        // When the read operations slowing down, profile the following 10 round and output the report to 'profile.pb'.
        if start_time.elapsed() > Duration::from_millis(40) && first_slow_round.is_none() {
            println!("Start profiling for slow round");
            first_slow_round = Some(round);
            profiler.reset();
        }
        if first_slow_round.unwrap_or(usize::MAX) == round.saturating_sub(10) {
            print!("Writing Profiling report write to 'profile.pb'... ");
            profiler.report_then_reset("profile.pb");
            println!("Done.")
        }

        round += 1;
    }
}

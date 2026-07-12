#![allow(missing_docs)]

use rand::RngExt;
use rand::distr::{
    Distribution,
    SampleString,
};

#[derive(Debug, Clone, Hash)]
struct Log {
    uuid: String,
    pref: usize,
    tags: Vec<u64>,
}

impl Log {
    fn random() -> Self {
        let mut rng = rand::rng();
        let uuid = rand::distr::Alphanumeric.sample_string(&mut rng, 10);
        let pref = rng.random_range(0..50);
        let tags = rand::distr::StandardUniform.sample_iter(&mut rng).take(4).collect();
        Log { uuid, pref, tags }
    }
}

#[test]
fn add_then_test() {
    let mut logs = Vec::with_capacity(100);
    let mut signs = bitfunnel::optimal(100, 0.05);

    for i in 0..100 {
        let log = Log::random();
        let mut sign = signs.sign_mut(i);
        sign.add(&("uuid", &log.uuid));
        sign.add(&("pref", &log.pref));
        for tag in &log.tags {
            sign.add(&("tags", tag));
        }
        logs.push(log);
    }

    for (i, log) in logs.into_iter().enumerate() {
        let sign = signs.sign(i as u64);
        assert!(sign.test(&("uuid", log.uuid)));
        assert!(sign.test(&("pref", log.pref)));
        for tag in &log.tags {
            assert!(sign.test(&("tags", tag)));
        }
    }
}

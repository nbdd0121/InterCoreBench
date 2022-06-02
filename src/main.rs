mod affinity;
mod semaphore;

use semaphore::Semaphore;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TEST_PERIOD: Duration = Duration::from_secs(5);
const TEST_INTERVAL: Duration = Duration::from_secs(1);

fn run_sync(down: &Semaphore, up: &Semaphore, cancel: &AtomicBool) -> usize {
    let mut count = 0;
    while !cancel.load(Ordering::Relaxed) {
        down.down();
        count += 1;
        up.up();
    }
    count
}

fn test_sync(core1: usize, core2: usize) {
    let cancel = Arc::new(AtomicBool::new(false));
    let s1 = Arc::new(Semaphore::new(1));
    let s2 = Arc::new(Semaphore::new(0));

    print!(
        "Testing latency between logical core {} and {}... ",
        core1, core2
    );

    let t1 = {
        let cancel = cancel.clone();
        let s1 = s1.clone();
        let s2 = s2.clone();
        std::thread::spawn(move || {
            affinity::set_affinity(core1);
            s1.down();
            let start = Instant::now();
            let count = run_sync(&s1, &s2, &cancel);
            let elapsed = start.elapsed();
            (count, elapsed)
        })
    };
    let t2 = {
        let cancel = cancel.clone();
        let s1 = s1.clone();
        let s2 = s2.clone();
        std::thread::spawn(move || {
            affinity::set_affinity(core2);
            s1.up_n(2);
            run_sync(&s2, &s1, &cancel)
        })
    };
    std::thread::sleep(TEST_PERIOD);
    cancel.store(true, Ordering::Relaxed);
    s1.up_n(100000);
    s2.up_n(100000);
    let (count, elapsed) = t1.join().unwrap();
    t2.join().unwrap();
    let rtt = elapsed / (count as u32);
    println!("{:?} ({} synchronisations per {:?})", rtt, count, elapsed);
}

fn main() {
    let cores = affinity::get_cores().unwrap();
    for i in 0..cores.len() {
        for j in (i + 1)..cores.len() {
            test_sync(cores[i], cores[j]);
            std::thread::sleep(TEST_INTERVAL);
        }
    }
}

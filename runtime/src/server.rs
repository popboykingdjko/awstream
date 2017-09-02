//! The main entrance for server functionality.

use super::{AsCodec, AsDatumType};
use chrono;
use chrono::{DateTime, TimeZone};
use futures::{Future, Stream};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;

fn time_diff_in_ms<Tz: TimeZone>(a: DateTime<Tz>, b: DateTime<Tz>) -> f64 {
    (a.timestamp() as f64 - b.timestamp() as f64) * 1000.0 +
        (a.timestamp_subsec_millis() as f64 - b.timestamp_subsec_millis() as f64)
}

/// Run the server. The server listens for new connections, parses input, and
/// prints performance statistics (latency, accuracy, etc).
///
/// The function will block until the server is shutdown.
pub fn server(port: u16) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = ([0, 0, 0, 0], port).into();
    let listener = TcpListener::bind(&addr, &handle).unwrap();

    // Accept all incoming sockets
    let server = listener.incoming().for_each(move |(socket, _)| {
        let transport = socket.framed(AsCodec::default());

        let process_connection = transport.for_each(|as_datum| {
            match as_datum.datum_type() {
                AsDatumType::Live(level) => {
                    let now = chrono::Utc::now();
                    let latency = time_diff_in_ms(now, as_datum.ts);
                    info!("level: {}, latency: {:.1} ms", level, latency);
                }
                _ => {}
            }
            Ok(())
        });

        // Spawn a new task dedicated to processing the connection
        handle.spawn(process_connection.map_err(|_| ()));
        Ok(())
    });

    // Open listener
    core.run(server).unwrap();
}

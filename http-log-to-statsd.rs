extern crate rustc_serialize;
extern crate docopt;
extern crate cadence;

use docopt::Docopt;
use cadence::prelude::*;
use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
use std::net::UdpSocket;

#[derive(Debug, RustcDecodable)]
struct Options {
    flag_v: isize,
    flag_listen: String,
    flag_statsd: String,
    flag_prefix: String,
}

fn main() {
        let usage = format!("
Usage:
  http-log-to-statsd [-h | --help] [-v...] [--listen=<listen>] [--statsd=<server>] [--prefix=<prefix>]

Options:
  -h --help                Show this screen.
  -v                       Increase verbosity.
  --listen=<listen>        Address and port number to listen on [default: 127.0.0.1:6666]
  --statsd=<server>        Address and port number of statsd server [default: 127.0.0.1:{}]
  --prefix=<prefix>        Statsd prefix for metrics [default: http.request]
", DEFAULT_PORT);

    let options: Options = Docopt::new(usage)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if options.flag_v > 1 { println!("{:?}", options) }

    let verbose = options.flag_v;

    let statsd = StatsdClient::<UdpMetricSink>::from_udp_host(options.flag_prefix.as_str(), options.flag_statsd.as_str()).unwrap();

    // Read from webserver and accumulate stats
    let socket = UdpSocket::bind(options.flag_listen.as_str()).unwrap();
    let mut buf = [0; 512];
    loop {
        if let Ok((amt, _/*src*/)) = socket.recv_from(&mut buf) {
            if let Ok(line) = std::str::from_utf8(&buf[0..amt]).map(|s| s.to_string()) {
                if verbose > 1 { println!("{}", line) }
                // <190>Sep  3 15:40:50 deck nginx: http GET 200 751 498 0.042 extra.suffix
                let line = if line.len() > 1 && line.chars().nth(0).unwrap() == '<' { // Strip off syslog gunk, if it exists
                    if let Some(start_byte) = line.find(": http").map(|l|l+2) {
                        std::str::from_utf8(&line.as_bytes()[start_byte..]).unwrap_or(line.as_str()).to_string()
                    } else { line }
                } else { line };
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 6 {
                    let empty_string = ""; // seriously, rust?
                    let (scheme, method, status, request_bytes, response_bytes, response_time_ms, suffix) = (fields[0], fields[1].to_lowercase(), fields[2], fields[3], fields[4], fields[5], fields.get(6).unwrap_or(&empty_string));
                    if verbose > 1 { println!("{},{},{},{},{},{},{}", scheme, method, status, request_bytes, response_bytes, response_time_ms, suffix) }

                    let name = |name: &str| { [name, suffix].concat() };

                    let _ = statsd.incr(&name(&scheme));
                    let _ = statsd.incr(&name(&method));
                    let _ = statsd.incr(&name(status));
                    let _ = statsd.incr(&name(&format!("{}xx", status.chars().nth(0).unwrap_or('X'))));

                    let _ = statsd.time(&name("request_bytes"),    request_bytes   .parse::<u64>().unwrap_or(0)); // looks wrong, but times get averaged, which is correct for bytes.
                    let _ = statsd.time(&name("response_bytes"),   response_bytes  .parse::<u64>().unwrap_or(0));
                    let _ = statsd.time(&name("response_time_ms"), if response_time_ms.contains('.') { (response_time_ms.parse::<f64>().unwrap_or(0.0) * 1000.0) as u64 } // ngingx
                                                                   else                              { response_time_ms.parse::<u64>().unwrap_or(0) });               // apache
                    let _ = statsd.incr(&name("requests"));
                } else if verbose > 0 { println!("!{}", line) }
            }
        }
    }
}


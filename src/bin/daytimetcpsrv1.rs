
use std::io::prelude::*;
use std::net::TcpListener;
 
extern crate chrono;
use chrono::prelude::*;

use docopt::Docopt;

use serde::Deserialize;

const USAGE: &'static str = "
Servidor de hora

Usage:
  daytimetcpsrc <ip> <porta>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_ip: String,
    arg_porta: u16,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let listener = TcpListener::bind(format!("{}:{}", args.arg_ip, args.arg_porta)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let today = Utc::now();
                let peer_addr = stream.peer_addr().unwrap();
                writeln!(stream, "{}", today.to_rfc2822()).unwrap();
                println!("{}", peer_addr.to_string());
            },
            Err(e) => {
                println!("Error: {}", e);
            },
        }
    }

    drop(listener);
}

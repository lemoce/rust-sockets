
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
        let today = Utc::now();
        writeln!(stream.unwrap(), "{}", today.to_rfc2822()).unwrap();
    }

}

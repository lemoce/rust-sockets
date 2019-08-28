

use std::io::{BufReader, Read};
use std::net::TcpStream;
use std::str;

use docopt::Docopt;

use serde::Deserialize;

const USAGE: &'static str = "
Pegar a hora de um servidor

Usage:
  daytimetcpcli <ip>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_ip: String,
}
 
fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let stream = TcpStream::connect(args.arg_ip).unwrap();
    stream.set_nonblocking(false).unwrap();
    let mut reader = BufReader::new(stream);

    let mut buffer = [0u8; 1024];
    let _r = reader.read(&mut buffer).unwrap();

    print!("{}", str::from_utf8_mut(&mut buffer).unwrap());
}

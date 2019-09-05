extern crate libc;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;

use docopt::Docopt;
use serde::Deserialize;


const USAGE: &'static str = "
Servidor de echo deixando processos zumbis

Usage:
  tcpserv01 <ip> <porta>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_ip: String,
    arg_porta: u16,
}

fn handle_stream(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0u8; 1028];

    while stream.read(&mut buffer)? != 0 {
        write!(stream, "{}", str::from_utf8(&buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?)?;
    }

    Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let listener = TcpListener::bind(format!("{}:{}", args.arg_ip, args.arg_porta)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if unsafe { libc::fork() } == 0 {

                    drop(listener);
                    handle_stream(&mut stream).unwrap();

                    std::process::exit(0);
                }
                drop(stream);
            },
            Err(e) => {
                println!("Error: {:?}", e);
            },
        }
    }

}

extern crate libc;
extern crate nix;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;

use docopt::Docopt;
use nix::sys::signal::{signal, Signal, SigHandler};
use serde::Deserialize;


const USAGE: &'static str = "
Servidor de echo com o tratamento dos processos filhos

Usage:
  tcpserv04 <ip> <porta>

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

extern fn sig_chld(_signo: libc::c_int) {
    let status = 0 as *mut libc::c_int;

    unsafe {
        let mut pid = libc::waitpid(-1, status, libc::WNOHANG);

        while pid > 0 {
            println!("child finished!");
            pid = libc::waitpid(-1, status, libc::WNOHANG);
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let listener = TcpListener::bind(format!("{}:{}", args.arg_ip, args.arg_porta)).unwrap();

    let sig_handler = SigHandler::Handler(sig_chld);
    unsafe { signal(Signal::SIGCHLD, sig_handler) }.unwrap();

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

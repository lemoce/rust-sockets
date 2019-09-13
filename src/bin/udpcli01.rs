use std::io::{BufRead, Write};
use std::net::UdpSocket;

use docopt::Docopt;
use serde::Deserialize;

const USAGE: &'static str = "
Cliente para um socket UDP

Usage:
  updcli01 --bind <bind-ip> --connect <dest-ip>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_bind_ip: String,
    arg_dest_ip: String,
}

fn dg_cli(sock: &mut UdpSocket) -> std::io::Result<()> {
    let mut buffer = String::new();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    
    let mut console_reader = stdin.lock();
    let mut console_writer = stdout.lock();

    while let Some(nread) = console_reader.read_line(&mut buffer).ok() {
        if nread != 0 {
            sock.send(buffer.as_bytes())?;

            sock.recv_from(unsafe { buffer.as_bytes_mut() })?;
            
            console_writer.write(buffer.as_bytes())?;
        } else {
            break;
        }

        buffer.clear();
    }

    Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut sock = UdpSocket::bind(args.arg_bind_ip).unwrap();

    // já filtra a conexão para não receber de vários clientes
    sock.connect(args.arg_dest_ip).unwrap();

    dg_cli(&mut sock).unwrap();

}

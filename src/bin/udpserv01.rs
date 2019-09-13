use std::net::UdpSocket;
use std::str;

use docopt::Docopt;
use serde::Deserialize;


const USAGE: &'static str = "
Servidor de echo escrito para protocolo UDP

Usage:
  udpserv01 <ip> <porta>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_ip: String,
    arg_porta: u16,
}

fn handle_stream(sock: &mut UdpSocket) -> std::io::Result<()> {
    let mut buffer = [0u8; 1024];

    while let Some(pair) = sock.recv_from(&mut buffer).ok() {
        if pair.0 > 0 {
            for ch in &mut buffer[pair.0 ..] {
                *ch = 0u8;
            }

            sock.send_to(&buffer, &pair.1)?;
        }
        else { break; }

    }

    Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut sock = UdpSocket::bind(format!("{}:{}", args.arg_ip, args.arg_porta)).unwrap();

    handle_stream(&mut sock).unwrap();

}

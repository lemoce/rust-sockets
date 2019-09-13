
use std::io::{BufRead, Write};
use std::net::{UdpSocket, ToSocketAddrs};
use std::str;

use docopt::Docopt;
use serde::Deserialize;

const USAGE: &'static str = "
Cliente para um socket UDP ignorando respostas de peers diferentes

Usage:
  updcli01_ignoring --bind <bind-ip> --connect <dest-ip>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_bind_ip: String,
    arg_dest_ip: String,
}

fn dg_cli(sock: &mut UdpSocket, peer: &String) -> std::io::Result<()> {
    let server_addr = peer.to_socket_addrs().unwrap().next().unwrap();
    
    let mut recv_line = [0u8; 1024];
    let mut send_line = String::new();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    
    let mut console_reader = stdin.lock();
    let mut console_writer = stdout.lock();

    while let Some(nread) = console_reader.read_line(&mut send_line).ok() {
        if nread != 0 {
            sock.send_to(send_line.as_bytes(), server_addr)?;

            let (_serv_nread, peer_addr) = sock.recv_from(&mut recv_line)?;
            if peer_addr != server_addr {
                println!("reply from {} ignored", peer_addr);
            } else {
                write!(console_writer, "{}", str::from_utf8(&recv_line).unwrap())?;
            }

        } else {
            break;
        }

        send_line.clear();
    }

    Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut sock = UdpSocket::bind(args.arg_bind_ip).unwrap();
    // sock.connect(args.arg_dest_ip).unwrap();

    dg_cli(&mut sock, &args.arg_dest_ip).unwrap();

}

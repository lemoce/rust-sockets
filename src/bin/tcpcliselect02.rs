extern crate nix;

use std::io::{BufReader, BufRead, Write};
use std::net::TcpStream;
use std::str;
use std::os::unix::io::AsRawFd;

use docopt::Docopt;
use nix::sys::select;
use nix::sys::socket;
use serde::Deserialize;

const USAGE: &'static str = "
Pegar a hora de um servidor

Usage:
  tcpcliselect02 <ip>

Options:
  -h --help     Mostra essa tela
"; 

#[derive(Debug, Deserialize)]
struct Args {
    arg_ip: String,
}

fn echo_cli(sock: &mut TcpStream) -> nix::Result<()> {
    let stdin = std::io::stdin();
    let mut client_reader = stdin.lock();

    let stdout = std::io::stdout();
    let mut client_writer = stdout.lock();

    let mut rset = select::FdSet::new();

    let cloned_sock = sock.try_clone().unwrap();
    let mut server_reader = BufReader::new(cloned_sock);

    let mut buffer = String::new();

    let mut stdin_eof = false;

    rset.clear();
    
    loop {

        if !stdin_eof {
            rset.insert(stdin.as_raw_fd());
        }
        
        rset.insert(sock.as_raw_fd());

        select::select(None, Some(&mut rset), None, None, None)?;

        if rset.contains(sock.as_raw_fd()) {
            match server_reader.read_line(&mut buffer) {
                Ok(0) => {
                    if stdin_eof {
                        return Ok(());
                    } else {
                        println!("Server terminou prematuramente.");
                        std::process::exit(1);
                    }
                },
                Ok(_n) => {
                    write!(client_writer, "{}", buffer).unwrap();
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    std::process::exit(1);
                },
            }
            
        }

        if rset.contains(stdin.as_raw_fd()) {
            match client_reader.read_line(&mut buffer) {
                Ok(0) => {
                    stdin_eof = true;
                    socket::shutdown(sock.as_raw_fd(), socket::Shutdown::Write)?;
                    rset.remove(stdin.as_raw_fd());
                    continue;
                },
                Ok(_n) => {
                    write!(sock, "{}", buffer).unwrap();
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    std::process::exit(1);
                },
                
            }
        }

        buffer.clear();

    }
    
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut stream = TcpStream::connect(args.arg_ip).unwrap();

    echo_cli(&mut stream).unwrap();

}

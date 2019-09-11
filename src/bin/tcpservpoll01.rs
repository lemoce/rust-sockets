extern crate nix;

use std::io::{BufReader, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd};
use std::str;

use docopt::Docopt;
use nix::poll;
use nix::unistd::{sysconf, SysconfVar};
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

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    
    let mut clients: Vec<poll::PollFd> = Vec::new();
    let mut client_socks: Vec<Box<TcpStream>> = Vec::new();

    let listener = TcpListener::bind(format!("{}:{}", args.arg_ip, args.arg_porta)).unwrap();
    clients.insert(0, poll::PollFd::new(listener.as_raw_fd(), poll::PollFlags::POLLRDNORM));


    loop {
        let mut nready = poll::poll(clients.as_mut_slice(), -1).unwrap();

        if clients[0].revents().unwrap().intersects(poll::PollFlags::POLLRDNORM) {
            let new_client = listener.accept().unwrap().0;
            let client_fd = new_client.as_raw_fd();

            if (sysconf(SysconfVar::OPEN_MAX).unwrap().unwrap() as usize) > clients.len() {
                clients.push(poll::PollFd::new(client_fd, poll::PollFlags::POLLRDNORM));

                // por algum raios o drop era executado ao final do if de accept,
                // dessa forma foi criado um boxed para não executar o drop
                client_socks.push(Box::new(new_client));
            } else {
                println!("too many clients");
                std::process::exit(1);
            }

            nready = nready - 1;
            if nready <= 0 {
                continue;
            }
        }
        
        for id in 1 .. clients.len() {

            let client = match clients.get_mut(id) {
	        Some(client) => client,
		None => break,
	    };

            
            if client.revents().unwrap().intersects(poll::PollFlags::POLLRDNORM | poll::PollFlags::POLLERR) {
                let mut buffer = String::new();

                let client_socket = client_socks.get(id-1).unwrap();

                // o clone foi feito para nao mover o tipo Box
                let client_reader = client_socket.try_clone().unwrap();
                let mut reader = BufReader::new(client_reader);

                let mut client_writer = client_socket.try_clone().unwrap();

                match reader.read_line(&mut buffer) {
                    Ok(0) => {
                        println!("Close client");
                        let _gomi = clients.remove(id);
                        let _gomi2 = client_socks.remove(id-1);
                    },
                    Ok(_) => {
                        write!(client_writer, "{}", buffer).unwrap();
                    }
                    Err(e) => {
                        if nix::errno::from_i32(nix::errno::errno()) == nix::errno::Errno::ECONNRESET {
                            println!("Connection reset");
                            let _gomi = clients.remove(id);
                            let _gomi2 = client_socks.remove(id-1);
                        } else {
                            println!("read error: {:?}", e);
                        }
                    }
                }

                buffer.clear();
                
                nready = nready - 1;
                if nready <= 0 {
                    break;
                }
                
            }

            
        }
    }
    
}

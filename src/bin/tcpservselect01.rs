extern crate nix;

use std::io::{BufReader, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use std::str;

use docopt::Docopt;
use nix::sys::select;
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
    
    let listener = TcpListener::bind(format!("{}:{}", args.arg_ip, args.arg_porta)).unwrap();

    let mut clients: Vec<TcpStream> = Vec::new();

    let mut allset = select::FdSet::new();
    allset.clear();

    allset.insert(listener.as_raw_fd());

    loop {
        let mut rset = allset.clone();
        println!("Bloqueado no select");
        let mut nready = select::select(None, Some(&mut rset), None, None, None).unwrap();
        println!("Liberado no select");

        
        if rset.contains(listener.as_raw_fd()) {
            let new_client = listener.accept().unwrap().0;
            let client_fd = new_client.as_raw_fd();

            if select::FD_SETSIZE > clients.len() {
                println!("Recebi um cliente");
                clients.push(new_client);
            } else {
                println!("too many clients");
                std::process::exit(1);
            }

            allset.insert(client_fd);
            
            nready = nready - 1;
            if nready <= 0 {
                continue;
            }
        }

        println!("antes do for");
        for id in 0 .. clients.len() - 1  {

            let client = clients.get_mut(id).unwrap();
            
            println!("Clients.iter_mut");

            let mut buffer = String::new();
            if rset.contains(client.as_raw_fd()) {

                let client_socket = client.try_clone().unwrap();
                let mut reader = BufReader::new(client_socket);

                match reader.read_line(&mut buffer) {
                    Ok(0) => {
                        println!("Close client");
                        let client_fd = client.as_raw_fd();
                        
                        drop(client);
                        allset.remove(client_fd);
                    },
                    Ok(_) => {
                        println!("Received => ({})", buffer);
                        write!(*client, "{}", buffer).unwrap();
                    }
                    Err(e) => {
                        println!("Some strange error: {:?}", e);
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
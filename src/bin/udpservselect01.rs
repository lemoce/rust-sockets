extern crate libc;
extern crate nix;

use std::io::prelude::*;
use std::net::{UdpSocket, TcpStream, ToSocketAddrs };
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::str;

use docopt::Docopt;
use nix::sys::socket;
use nix::sys::select;
use nix::sys::signal::{signal, Signal, SigHandler};
use nix::unistd;
use serde::Deserialize;


const USAGE: &'static str = "
Servidor que trata TCP e UDP ao mesmo tempo

Usage:
  udpservselect01 <ip> <porta>

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

fn handle_dgram(sock: &mut UdpSocket) -> std::io::Result<()> {
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

extern fn sig_chld(_signo: libc::c_int) {
    let status = 0 as *mut libc::c_int;

    unsafe {
        let mut pid = libc::waitpid(-1, status, libc::WNOHANG);

        while pid > 0 {
            pid = libc::waitpid(-1, status, libc::WNOHANG);
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let listen_addr = format!("{}:{}", args.arg_ip, args.arg_porta)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let listen_fd =
        socket::socket(
            socket::AddressFamily::Inet,
            socket::SockType::Stream,
            socket::SockFlag::empty(),
            socket::SockProtocol::Tcp)
        .unwrap();
    socket::setsockopt(listen_fd,
                       socket::sockopt::ReuseAddr,
                       &true
    ).unwrap();
    socket::bind(listen_fd,
                 &socket::SockAddr::new_inet(
                     socket::InetAddr::from_std(&listen_addr)
                 )
    ).unwrap();
    socket::listen(listen_fd, 512).unwrap();

    let mut udp_sock = UdpSocket::bind(&listen_addr).unwrap();

    let sig_handler = SigHandler::Handler(sig_chld);
    unsafe { signal(Signal::SIGCHLD, sig_handler) }.unwrap();

    let mut allset = select::FdSet::new();
    allset.clear();

    allset.insert(listen_fd);
    allset.insert(udp_sock.as_raw_fd());

    loop {
        let mut rset = allset.clone();
        let nready = select::select(None, Some(&mut rset), None, None, None).unwrap();

        if nready < 0 {
            // implementar o EINTR
        }

        if rset.contains(listen_fd) {
            let client_fd = socket::accept(listen_fd).unwrap();
            if unsafe { libc::fork() } == 0 {
                unistd::close(listen_fd).unwrap();
                let mut client_sock = unsafe { TcpStream::from_raw_fd(client_fd) };
                handle_stream(&mut client_sock).unwrap();
                std::process::exit(0);
            } else {
                unistd::close(client_fd).unwrap();
            }
        }

        if rset.contains(udp_sock.as_raw_fd()) {
            handle_dgram(&mut udp_sock).unwrap();
        }
    }
    
}

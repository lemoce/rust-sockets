extern crate libc;

mod lemoce {

    mod sockets {

        use std::os::unix::io::RawFd;

        #[derive(Clone, Debug)]
        struct Socket {
            fd: RawFd,
        }

        impl Socket {
            fn new(raw_fd: RawFd) -> Socket {
                Socket { fd: raw_fd }
            }
        }

        pub enum Family {
            Inet,
            Inet6,
            Unspec,
        }

        impl Family {
            fn libc_const(&self) -> i32 {
                match *self {
                    Family::Inet => libc::AF_INET,
                    Family::Inet6 => libc::AF_INET6,
                    Family::Unspec => libc::AF_UNSPEC,                    
                }
            }
        }
        
        pub enum Protocol {
            Tcp,
            Udp,
            Sctp,
        }

        impl Protocol {
            fn libc_const(&self) -> i32 {
                match *self {
                    Protocol::Tcp => libc::IPPROTO_TCP,
                    Protocol::Udp => libc::IPPROTO_UDP,
                    Protocol::Sctp => libc::IPPROTO_SCTP,
                }
            }
        }
        
        fn socket(family: Family, proto: Protocol) -> Option<Socket> {
            let fd = unsafe {
                libc::socket(family.libc_const(), libc::SOCK_SEQPACKET, proto.libc_const())
            };
            if fd < 0 {
                None
            } else {
                Some ( Socket::new(fd) )
            }
        }

        
    }
}

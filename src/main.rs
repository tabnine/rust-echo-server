use anyhow::{Result, anyhow};
use std::{io, thread};
use std::net::{TcpListener, SocketAddr, SocketAddrV4, Ipv4Addr, TcpStream};
use socket2::{Socket, Domain, Type};
use std::io::{BufReader, Write, BufRead};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "rust-echo-server")]
struct Opt {
    #[structopt(short = "-p", long = "port", default_value = "49152")]
    port: u16,

    #[structopt(short = "-b", long = "backlog", default_value = "0")]
    backlog: i32,
}

fn main() {
    let opt = Opt::from_args();

    println!("Echo server starting. port = {}, backlog = {}", opt.port, opt.backlog);

    listen(&opt).unwrap();

    io::stdin().lock().lines().next();
}

fn listen(opt: &Opt) -> Result<()> {
    let listener = create_listener(opt);
    match listener {
        Ok(listener) => {
            accept_connections(listener);
            Ok(())
        }
        Err(err) => {
            Err(anyhow!("Failed to listen on port {}, {:?}", opt.port, err))
        }
    }
}

fn create_listener(opt: &Opt) -> io::Result<TcpListener> {
    let socket = Socket::new(Domain::ipv4(), Type::stream(), None)?;

    let socket_address = SocketAddr::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, opt.port));
    socket.bind(&socket_address.into())?;

    socket.listen(opt.backlog)?;

    Ok(socket.into_tcp_listener())
}

fn accept_connections(listener: TcpListener) {
    let _thread_result = thread::Builder::new()
        .name("Local Connections Acceptor".to_string())
        .spawn(move || loop {
            let accepted = listener.accept();
            match accepted {
                Err(e) => eprintln!("Error accepting connection {}", e),
                Ok((client, address)) => {
                    let r = client.set_nodelay(true);
                    if let Err(e) = r {
                        eprintln!(
                            "Failed to set nodelay on incoming connection {} {}",
                            address, e
                        );
                    }

                    println!("Got new local client connection {}", address);
                    handle_connection(client, address);
                }
            }
        });
}

fn handle_connection(mut client: TcpStream, address: SocketAddr) {
    let _thread_result = thread::Builder::new()
        .name(format!("Local Connection Handler {}", address))
        .spawn(move || {
            let reader_client = client
                .try_clone()
                .expect("Failed to clone socket for reading");
            let reader = BufReader::new(reader_client);

            for line in reader.lines() {
                match line {
                    Err(e) => {
                        eprintln!("Error reading from socket {:?}", e);
                        break;
                    }
                    Ok(line) => {
                        if let Err(e) = client.write_all(format!("{}\n", line).as_bytes()) {
                            eprintln!("Failed to write to socket {:?}", e);
                            break;
                        }
                    }
                }
            }
        });
}

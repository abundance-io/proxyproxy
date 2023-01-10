pub mod config;
pub mod proxy;
pub mod tcp;
use std::{
    error::Error,
    io::{BufRead, BufReader, LineWriter, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};
use tcp::TcpConnection;

fn to_SocketAddr(ip: [u8; 4], port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener_socket = to_SocketAddr([127, 0, 0, 1], 1973);
    let listener = TcpListener::bind(listener_socket)?;
    for stream in listener.incoming() {
        let stream = stream?;
        let mut conn = TcpConnection::from_stream(stream)?;
        // loop {
        //     let reccieved = conn.read()?;
        //     conn.write(b"thanks for that");
        //     println!("{:?}", reccieved)
        // }
    }
    Ok(())
}

fn forward(stream: TcpStream, client: SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut client_stream = TcpStream::connect(client)?;
    // let mut reader = BufReader::new(&mut stream)
    // let reccieved = reader.fill_buf()?.to_vec()
    let mut reader = BufReader::new(stream);
    let mut writer = LineWriter::new(client_stream);
    writer.write(reader.fill_buf()?);
    Ok(())
}

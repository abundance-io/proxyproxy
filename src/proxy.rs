use crate::{
    config,
    tcp::{to_socket_addr, TcpConnCollection, TcpConnection, TcpInstance, TcpListenerCollection},
};
use std::{
    error::Error,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

pub fn start_proxy_from_config(config: config::Config) {
    for app in config.app.into_iter() {
        let app_instance = thread::spawn(move || start_proxy_from_app(app.clone()).unwrap());
    }
}
pub fn start_proxy_from_app(app: config::App) -> Result<(), Box<dyn Error>> {
    let server_addresses = app
        .ports
        .clone()
        .into_iter()
        .map(|port| to_socket_addr([127, 0, 0, 1], port))
        .collect();
    let listener_collection = TcpListenerCollection::new(server_addresses);
    for stream in listener_collection.incoming() {
        println!("{:?}", stream);
        let mut stream_conn = TcpConnection::from_stream(stream)?;
        let targets = app.targets.clone();
        let server_thread = thread::spawn(move || {
            let mut client_conn = stream_conn;
            let mut dest_conn = TcpConnCollection::from(targets);
            handle_conn(&mut client_conn, &mut dest_conn);
        });
    }
    Ok(())
}
pub fn start_proxy(ip: [u8; 4], port: u16) -> Result<(), Box<dyn Error>> {
    let server_address = to_socket_addr(ip, port);
    let server_conn = TcpListener::bind(server_address)?;
    for stream in server_conn.incoming() {
        let stream = stream?;
        let mut stream_conn = TcpConnection::from_stream(stream)?;
        let server_thread = thread::spawn(move || {
            let mut client_conn = stream_conn;
            let mut dest_conn = TcpConnection::new([127, 0, 0, 1], 1972).unwrap();
            handle_conn(&mut client_conn, &mut dest_conn)
        });
        server_thread.join().unwrap()
    }
    let server_details = "127.0.0.1:80";
    let server: SocketAddr = server_details
        .parse()
        .expect("Unable to parse socket address");
    println!("{:?}", server);
    Ok(())
}

pub fn handle_conn<T: TcpInstance, U: TcpInstance>(client_conn: &mut T, dest_conn: &mut U) -> () {
    loop {
        let client_data = client_conn.read().unwrap();
        if !client_data.is_empty() {
            for data in client_data {
                dest_conn.write(&data).unwrap();
            }
        }
        let server_data = dest_conn.read().unwrap();
        if !server_data.is_empty() {
            for data in server_data {
                client_conn.write(&data).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn base_proxy_works() {
        start_proxy([127, 0, 0, 1], 1973);
    }

    fn proxy_loads_from_config() {
        start_proxy_from_config(config::get_config("./config.json"));
    }
}

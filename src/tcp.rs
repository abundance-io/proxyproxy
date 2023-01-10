use std::{
    error::Error,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub fn to_socket_addr(ip: [u8; 4], port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
}

pub trait TcpInstance {
    //the return type of the read is generic as it could be a single buffer or a list of buffers
    fn read(&mut self) -> Result<Vec<Vec<u8>>, Box<dyn Error>>;
    fn write(&mut self, buffer: &[u8]) -> Result<(), Box<dyn Error>>;
}

pub struct TcpConnection {
    //just a wrapper around TCP stream
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl TcpConnection {
    //listen and connect are builder functions
    pub fn new(ip: [u8; 4], port: u16) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(to_socket_addr(ip, port))?;

        let stream_clone = stream.try_clone()?;
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(stream_clone);
        Ok(Self { reader, writer })
    }

    pub fn from_stream(stream: TcpStream) -> Result<Self, Box<dyn Error>> {
        let stream_clone = stream.try_clone()?;
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(stream_clone);
        Ok(Self { reader, writer })
    }
}
impl TcpInstance for TcpConnection {
    fn read(&mut self) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let reccieved = self.reader.fill_buf()?.to_vec();
        self.reader.consume(reccieved.len());
        let mut recieved_coll = Vec::new();
        recieved_coll.push(reccieved);
        Ok(recieved_coll)
    }

    fn write(&mut self, buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        self.writer.write(buffer)?;
        self.writer.flush()?;
        Ok(())
    }
}

pub struct TcpConnCollection {
    connections: Vec<TcpConnection>,
    balance_count: usize,
}
impl From<Vec<u16>> for TcpConnCollection {
    fn from(ports: Vec<u16>) -> TcpConnCollection {
        let instances = ports
            .clone()
            .into_iter()
            .map(|port| TcpConnection::new([127, 0, 0, 1], port).unwrap())
            .collect();

        return TcpConnCollection {
            connections: instances,
            balance_count: 0,
        };
    }
}

impl From<Vec<String>> for TcpConnCollection {
    fn from(targets: Vec<String>) -> TcpConnCollection {
        let targets: Vec<SocketAddr> = targets
            .clone()
            .into_iter()
            .map(|target| target.parse().expect("could not parse address"))
            .collect();
        let connections = targets
            .iter()
            .map(|address| {
                TcpConnection::from_stream(TcpStream::connect(address).unwrap()).unwrap()
            })
            .collect();
        return TcpConnCollection {
            connections,
            balance_count: 0,
        };
    }
}

impl TcpInstance for TcpConnCollection {
    fn read(&mut self) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let messages: Vec<Vec<u8>> = self
            .connections
            .iter()
            .map(|conn| conn.read().unwrap().remove(0))
            .collect();

        Ok(messages)
    }

    fn write(&mut self, buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        if self.balance_count > self.connections.len() {
            self.balance_count = 0;
        } else {
            self.balance_count += 1;
            match self.connections[self.balance_count].write(buffer) {
                Ok(()) => {}
                Err(err) => {
                    let retry = self.write(buffer);
                }
            };
        }
        Ok(())
    }
}

pub struct TcpListenerCollection {
    listeners: Vec<TcpListener>,
}

impl TcpListenerCollection {
    pub fn new(addresses: Vec<SocketAddr>) -> Self {
        let listeners = addresses
            .into_iter()
            .map(|address| TcpListener::bind(address).unwrap())
            .collect();

        return Self { listeners };
    }

    pub fn incoming(self) -> impl Iterator<Item = TcpStream> {
        let (tx, rx): (Sender<TcpStream>, Receiver<TcpStream>) = mpsc::channel();
        for listener in self.listeners {
            let tx = tx.clone();
            let listener_thread = thread::spawn(move || {
                for stream in listener.incoming() {
                    let stream = stream.unwrap();
                    tx.send(stream);
                }
            });
        }

        let iter_func = std::iter::from_fn(move || {
            let data = rx.recv();
            match data {
                Ok(data) => Some(data),
                Err(err) => None,
            }
        });

        return iter_func;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parallel_read_write_from_tcp_connection() {
        let listener_socket = to_socket_addr([127, 0, 0, 1], 1973);
        let listener = TcpListener::bind(listener_socket).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let mut conn = TcpConnection::from_stream(stream).unwrap();
            loop {
                let reccieved = conn.read().unwrap();
                conn.write(b"thanks for that").unwrap();
                println!("{:?}", reccieved)
            }
        }
    }
}

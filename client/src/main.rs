use std::{fs::File, io::Read, net::{UdpSocket, IpAddr, Ipv4Addr, SocketAddr}, time::Duration};
use nb;
use std::io;

use ascii::AsciiStr;
use tftp::Message;
use message::{wrq, rrq, ack, data, error};

pub struct TftpClient {
    local_ip: IpAddr
}

impl TftpClient {
    pub fn new(local_ip: IpAddr) -> Self {
        Self {local_ip}
    }
    //I don't think that to copy file into vec is a good idea. 
    //Usually in such cases, buffers are used
    pub fn read_file(&mut self, filename: &str) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        let mut f = File::open(filename).unwrap();
        f.read_to_end(&mut buf).expect("can't read file");
        buf
    }

	fn socket(&mut self) -> io::Result<UdpSocket> {
       Ok(UdpSocket::bind(SocketAddr::new(self.local_ip, 0))
                            .expect("couldn't bind to address"))
    }

	fn connect(&mut self, socket: &mut UdpSocket, remote: SocketAddr) -> io::Result<()> {
        socket.connect(remote).expect("connect function failed");
        Ok(())
    }

	fn send(&mut self, socket: &mut UdpSocket, buffer: &[u8]) -> nb::Result<(), nb::Error<io::Error>> {
        socket.send(buffer).expect("couldn't send data");
        Ok(())
    }

	fn receive(&mut self, socket: &mut UdpSocket, buffer: &mut [u8]) 
        -> nb::Result<(usize, SocketAddr), nb::Error<io::Error>> {
        let (number_of_bytes, src_addr) = socket.recv_from(buffer)
                                            .expect("Didn't receive data");
        Ok((number_of_bytes, src_addr))                  
    }

	fn close(&mut self, socket: UdpSocket) -> io::Result<()> {
        todo!()
    }

	fn bind(&mut self, socket: &mut UdpSocket, local_port: u16) -> io::Result<()> {
        *socket = UdpSocket::bind(SocketAddr::new(self.local_ip, local_port))
                        .expect("couldn't bind to address");
        Ok(())
    }

	fn send_to(&mut self, socket: &mut UdpSocket, remote: SocketAddr, buffer: &[u8]) 
        -> nb::Result<(), nb::Error<io::Error>> {
        socket.send_to(buffer, remote).expect("couldn't send data");
        Ok(())
    }
}

fn main() {
    let mut TC = TftpClient::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let mut socket = TC.socket()?;
    TC.bind(&mut socket, 8081)?;

    let path = "read_from.txt";

    let packet: Vec<u8> = wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
        .unwrap()
        .into();    
    TC
        .send_to(&mut socket, 
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 69), 
            packet.as_slice())?;
    
    let mut file = TC.read_file("read_from.txt");
    //need to put slice creation in loop (0-511, 512 - 1023 and so on)
    let file_slice = if file.len() > 512 {
        &file[0..512]
    } else {
        &file[..]
    };

    let mut r_buf = [0; 516];
    //necessary to add break after several error messages
    loop {
        let (_number_of_bytes, src_addr) =
            TC.receive(&mut socket, &mut r_buf)?;
        let message = Message::try_from(&r_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(0) => {
                println!("receive ack message");
                TC.connect(&mut socket, src_addr)?;
                break;
            }

            _ => continue,
        }
    }

    let block_id = 1u16;
    let packet: Vec<u8> = data(block_id, file_slice).unwrap().into();

    loop {
        TC.send(&mut socket, packet.as_slice())?;
        TC.receive(&mut socket, &mut r_buf)?;
        let message = Message::try_from(&r_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(id) => {
                if id == block_id {
                    println!("receive ack message");
                    break;
                } else {
                    println!("wrong block id");
                    continue;
                };
            }
            _ => continue,
        }
    }
}

/*fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8081").expect("couldn't bind to address");
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let path = "read_from.txt";

    let packet: Vec<u8> = wrq(AsciiStr::from_ascii(path.as_bytes()).unwrap(), true)
        .unwrap()
        .into();
    socket
        .send_to(packet.as_slice(), "127.0.0.1:69")
        .expect("couldn't send data");

    let mut r_buf = [0; 516];
    //necessary to add break after several error messages
    loop {
        let (_number_of_bytes, src_addr) =
            socket.recv_from(&mut r_buf).expect("didn't receive data");
        let message = Message::try_from(&r_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(0) => {
                println!("receive ack message");
                socket.connect(src_addr).expect("connect function failed");
                break;
            }

            _ => continue,
        }
    }

    let mut buf = [0; 512];
    let mut f = File::open(path).unwrap();
    f.read(&mut buf).unwrap();
    let block_id = 1u16;
    let packet: Vec<u8> = data(block_id, &buf[..]).unwrap().into();

    loop {
        socket.send(packet.as_slice()).expect("couldn't send data");
        socket.recv(&mut r_buf).expect("didn't receive data");
        let message = Message::try_from(&r_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Ack(id) => {
                if id == block_id {
                    println!("receive ack message");
                    break;
                } else {
                    println!("wrong block id");
                    continue;
                };
            }
            _ => continue,
        }
    }
}*/

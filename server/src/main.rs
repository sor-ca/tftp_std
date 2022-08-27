use std::{
    fs::File,
    io::{Write, Read, Result},
    net::{UdpSocket, ToSocketAddrs},
    thread,
    time::{self, Duration},
};

use tftp::{FileOperation, Message};
use message::{ack, data, error};

pub struct TftpServer {
    socket: UdpSocket,
}

impl TftpServer {
    fn new<A: ToSocketAddrs>(socket_addr: A) -> Self {
        Self {
            socket: UdpSocket::bind(socket_addr)
                            .expect("couldn't bind to address")
        }        
    }

    //port 8080
    fn listen<A: ToSocketAddrs>(&mut self, socket_addr: A) -> Result<FileOperation> {
        loop { 
            let mut buf = [0; 516];   
            let (number_of_bytes, src_addr) = self.socket.recv_from(&mut buf).expect("didn't receive data");
            let filled_buf = &mut buf[..number_of_bytes]; 
            let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
            match message {
                Message::File {operation, ..} => {     
                    println!("receive request");
                    self.socket = UdpSocket::bind(socket_addr).expect("couldn't bind to address");
                    self.socket.connect(src_addr).expect("connect function failed");
                    let packet: Vec<u8> = ack(0).into();
                    self.socket.send(packet.as_slice()).expect("couldn't send data");
                    let file_operation = operation;
                    //let filename = path.as_str();
                    return Ok(file_operation);
                }
                _ => continue,
            }
        }

    }

    fn write(&mut self) -> Result<()> {
        let mut f = File::create("write_into.txt").unwrap();
        let mut vec = Vec::with_capacity(1024*1024);
    
        //necessary to add break after several error messages
        loop {
            let mut buf = [0; 516];
            let number_of_bytes = self.socket.recv(&mut buf).expect("didn't receive data");
            let filled_buf = &mut buf[..number_of_bytes];
            let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
            match message {
                Message::Data(block_id, data) => {
                    println!("receive data packet");
                    //dbg!(str::from_utf8(data.as_ref()).expect("can't read message"));
                    vec.extend_from_slice(data.as_ref());
    
                    let packet: Vec<u8> = ack(block_id).into();
                    thread::sleep(time::Duration::from_secs(1));
                    self.socket.send(packet.as_slice()).expect("couldn't send data");
                    if number_of_bytes < 516 {
                        break;
                    } else {
                        continue;
                    }            
                },
    
               _ => continue,
            }
        }
        f.write(vec.as_slice()).unwrap();
        Ok(())
    }

    fn read(&mut self) -> Result<()> {
        let mut vec: Vec<u8> = vec![];
        let mut f = File::open("read_from.txt").expect("can't open file");
        f.read_to_end(&mut vec).expect("can't read file");
        let mut i = 0;
        let mut j = 512;
        let mut vec_slice: &[u8];
        let mut block_id = 1u16;

        loop {
            vec_slice = if vec.len() > j {
                &vec[i..j]
            } else {
                &vec[i..]
            };
    
            let packet: Vec<u8> = data(block_id, vec_slice).unwrap().into();

            loop {
                self.socket.send(packet.as_slice()).expect("couldn't send data");
                let mut r_buf = [0; 516];
                let number_of_bytes = self.socket.recv(&mut r_buf).expect("didn't receive data");
                let filled_buf = &mut r_buf[..number_of_bytes];
                let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
                
                match message {
                    Message::Ack(id) => {
                        if id == block_id {
                            println!("receive ack message");
                            block_id += 1;
                            break;
                        } else {
                            println!("wrong block id");
                            continue;
                        };
                    }
                    _ => continue,
                }
            }
           
            if vec.len() <= j {
                println!("file came to end");
                break;
            }
            i += 512;
            j += 512;
        }
        Ok(())
        //todo!()
    }
}

fn main() {
    let mut server = TftpServer::new("127.0.0.1:69");
    server.socket
        .set_read_timeout(Some(Duration::from_secs(100)))
        .unwrap();
    let result = server.listen("127.0.0.1:8080").expect("no request");
    match result {
        FileOperation::Write => server.write().expect("server writing error"),
        FileOperation::Read => server.read().expect("server reading error"),
    };
}

/*fn main() {
    let mut socket = UdpSocket::bind("127.0.0.1:69").expect("couldn't bind to address");
    socket
        .set_read_timeout(Some(Duration::from_secs(100)))
        .unwrap();


    //necessary to add break after several error messages
    loop { 
        let mut buf = [0; 516];   
        let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("didn't receive data");
        let filled_buf = &mut buf[..number_of_bytes]; 
        let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
        match message {
            Message::File {
                operation: FileOperation::Write,
                ..
            } => {
                println!("receive wrq message");
                socket = UdpSocket::bind("127.0.0.1:8080").expect("couldn't bind to address");
                socket.connect(src_addr).expect("connect function failed");
                let packet: Vec<u8> = ack(0).into();
                socket.send(packet.as_slice()).expect("couldn't send data");
                break;
            }

            _ => continue,
        }
    }
    let mut f = File::create("write_into.txt").unwrap();
    let mut vec = Vec::with_capacity(1024*1024);
    
        //necessary to add break after several error messages
    loop {
        let mut buf = [0; 516];
        let number_of_bytes = self.socket.recv(&mut buf).expect("didn't receive data");
        let filled_buf = &mut buf[..number_of_bytes];
        let message = Message::try_from(&filled_buf[..]).expect("can't convert buf to message");
        match message {
            Message::Data(block_id, data) => {
                println!("receive data packet");
                //dbg!(str::from_utf8(data.as_ref()).expect("can't read message"));
                vec.extend_from_slice(data.as_ref());
    
                let packet: Vec<u8> = ack(block_id).into();
                thread::sleep(time::Duration::from_secs(1));
                self.socket.send(packet.as_slice()).expect("couldn't send data");
                if number_of_bytes < 516 {
                    break;
                } else {
                    continue;
                }            
            },
    
             _ => continue,
        }
    }
    f.write(vec.as_slice()).unwrap();            
}*/



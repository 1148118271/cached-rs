use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;


pub struct Cached {
    stream: TcpStream
}


impl Cached {
    pub fn connect() -> io::Result<Cached> {
        let mut stream = TcpStream::connect("127.0.0.1:9200")?;
        let mut status = [0; 3];
        stream.read(&mut status)?;
        if status.eq(&[b'2', b'0', b'0']) {
            stream.set_nonblocking(true)?;
            return Ok(Cached {
                stream
            })
        }
        Err(io::Error::new(ErrorKind::ConnectionAborted, "Connection fail"))
    }

    pub fn set(&mut self, key: &str, value: &str) -> io::Result<Vec<u8>> {
        let request = format!("set {} {}", key, value);
        self.stream.write_all(request.as_bytes())?;
        self.read()
    }

    pub fn get(&mut self, key: &str) -> io::Result<Vec<u8>> {
        let request = format!("get {}", key);
        self.stream.write_all(request.as_bytes())?;
        let result = self.read()?;
        if result.len() > 2 {
            let (_, r) = result.split_at(2);
            return Ok(r.to_vec())
        }
        return Ok(vec![])
    }

    pub fn remove(&mut self, key: &str) -> io::Result<Vec<u8>> {
        let request = format!("rm {}", key);
        self.stream.write_all(request.as_bytes())?;
        let result = self.read()?;
        if result.len() > 1 {
            let (_, r) = result.split_at(2);
            return Ok(r.to_vec())
        }
        return Ok(vec![])
    }

    fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = [0; 1024];
        let mut bufs = vec![];
        loop {
            match self.stream.read(&mut buf) {
                Ok(size) => {
                    if size <= 0 {
                        return Ok(bufs.to_vec())
                    }
                    bufs.extend_from_slice(&buf[..size])
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        if bufs.len() > 0 {
                            return Ok(bufs.to_vec())
                        }
                        continue
                    }
                    return Err(e)
                }
            }
        }

    }

}


#[test] fn t() {
    let mut cached = Cached::connect().unwrap();
    let result = cached.set("key", "value").unwrap();
    println!("{:?}", String::from_utf8(result));
    let result = cached.get("key").unwrap();
    println!("{:?}", String::from_utf8(result));
    let result = cached.remove("key").unwrap();
    println!("{:?}", String::from_utf8(result));
    let result = cached.get("key").unwrap();
    println!("{:?}", String::from_utf8(result));
}
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct LazyTcpStream {
    addr: String,
    stream: Option<TcpStream>,
    max_retries: u32,
    read_timeout: Duration,
    write_timeout: Duration,
    connect_timeout: Duration,
}

impl LazyTcpStream {
    pub fn new<A: ToSocketAddrs>(
        addr: A,
        max_retries: u32,
        read_timeout: Duration,
        write_timeout: Duration,
        connect_timeout: Duration,
    ) -> Self {
        LazyTcpStream {
            addr: addr
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .map(|addr| addr.to_string())
                .unwrap_or_default(),
            stream: None,
            max_retries,
            read_timeout,
            write_timeout,
            connect_timeout,
        }
    }

    fn connect(&mut self) -> std::io::Result<()> {
        for attempt in 0..=self.max_retries {
            match TcpStream::connect_timeout(
                &self
                    .addr
                    .parse()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?,
                self.connect_timeout,
            ) {
                Ok(stream) => {
                    stream.set_read_timeout(Some(self.read_timeout))?;
                    stream.set_write_timeout(Some(self.write_timeout))?;

                    stream.set_nonblocking(false)?;
                    self.stream = Some(stream);
                    return Ok(());
                }
                Err(e) if attempt == self.max_retries => return Err(e),
                Err(_) => {}
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Max connection retries reached",
        ))
    }

    fn ensure_connected(&mut self) -> std::io::Result<()> {
        if self.stream.is_none() {
            self.connect()?;
        }
        Ok(())
    }

    fn reconnect_if_needed(&mut self) -> std::io::Result<()> {
        if let Some(stream) = &mut self.stream {
            let mut buf = [0; 1];
            match stream.peek(&mut buf) {
                Err(e)
                    if e.kind() == std::io::ErrorKind::ConnectionReset
                        || e.kind() == std::io::ErrorKind::ConnectionAborted =>
                {
                    self.reconnect()
                }
                _ => Ok(()),
            }
        } else {
            self.connect()
        }
    }

    pub fn reconnect(&mut self) -> std::io::Result<()> {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
        self.connect()
    }
}

impl Read for LazyTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.ensure_connected()?;
        self.reconnect_if_needed()?;

        match self.stream.as_mut().unwrap().read(buf) {
            Ok(n) => Ok(n),
            Err(e)
                if e.kind() == std::io::ErrorKind::ConnectionReset
                    || e.kind() == std::io::ErrorKind::ConnectionAborted
                    || e.kind() == std::io::ErrorKind::BrokenPipe =>
            {
                let _ = self.stream.as_mut().unwrap().shutdown(Shutdown::Both);
                self.stream = None;
                self.reconnect_if_needed()?;
                self.stream.as_mut().unwrap().read(buf)
            }
            Err(e) => Err(e),
        }
    }
}

impl Write for LazyTcpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.ensure_connected()?;
        self.reconnect_if_needed()?;

        match self.stream.as_mut().unwrap().write(buf) {
            Ok(n) => Ok(n),
            Err(e)
                if e.kind() == std::io::ErrorKind::ConnectionReset
                    || e.kind() == std::io::ErrorKind::ConnectionAborted
                    || e.kind() == std::io::ErrorKind::BrokenPipe =>
            {
                let _ = self.stream.as_mut().unwrap().shutdown(Shutdown::Both);
                self.stream = None;
                self.reconnect_if_needed()?;
                self.stream.as_mut().unwrap().write(buf)
            }
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.ensure_connected()?;
        self.reconnect_if_needed()?;
        self.stream.as_mut().unwrap().flush()
    }
}

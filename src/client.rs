use crate::tls;
use log::info;
use rustls::Session;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
struct ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client Error")
    }
}

struct GeminiRequest {
    scheme: String,
    hostname: String,
    port: String,
    path: String,
    address: SocketAddr,
}

impl GeminiRequest {
    unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(
            (self as *const GeminiRequest) as *const u8,
            std::mem::size_of::<GeminiRequest>(),
        )
    }

    fn for_tcp(&self) -> String {
        format!(
            "{address}:{port}",
            address = self.hostname,
            port = self.port
        )
    }

    fn for_dns(&self) -> &String {
        &self.hostname
    }

    fn request(&self) -> String {
        format!(
            "{scheme}://{hostname}{path}\r\n",
            scheme = self.scheme,
            hostname = self.hostname,
            path = self.path,
        )
    }
}

pub struct GeminiClient {}

impl GeminiClient {
    pub fn new() -> Self {
        GeminiClient {}
    }

    fn resolve_hostname(&self, hostname: String) -> Result<SocketAddr, ClientError> {
        let address = hostname.to_socket_addrs();
        let mut address = match address {
            Ok(addr) => addr,
            Err(_) => return Err(ClientError {}),
        };

        Ok(address.next().unwrap())
    }

    fn parse_request(&self, request: String) -> Result<GeminiRequest, ClientError> {
        let mut request = request.trim().to_string();

        let scheme_index = request.find("://");
        let scheme = match scheme_index {
            Some(i) => {
                let scheme = request.drain(..i).collect();
                request = request.replacen("://", "", 1);
                scheme
            }
            None => String::from("gemini"),
        };

        let port_index = request.find(":");

        let (hostname, port) = match port_index {
            Some(i) => {
                let hostname = request.drain(..i).collect();
                request = request.replacen(":", "", 1);
                let port = request.find("/");
                let port = match port {
                    Some(i) => {
                        let port = request.drain(..i).collect();
                        request = request.replacen("/", "", 1);
                        port
                    }
                    None => request.drain(..).collect(),
                };
                (hostname, port)
            }
            None => {
                let hostname = match request.find("/") {
                    Some(i) => {
                        let hostname = request.drain(..i).collect();
                        hostname
                    }
                    None => request.drain(..).collect(),
                };
                let port = match scheme.as_str() {
                    "gemini" => String::from("1965"),
                    _ => String::from("0"),
                };
                (hostname, port)
            }
        };

        let path = request;
    //    info!("scheme: {}", scheme);
    //    info!("hostname: {}", hostname);
    //    info!("port: {}", port);
    //    info!("path: {}", path);

        let resolve = format!("{}:{}", hostname, port);
        let address = self.resolve_hostname(resolve).unwrap();

        Ok(GeminiRequest {
            scheme,
            hostname,
            port,
            path,
            address,
        })
    }

    pub fn goto_url(&self, url: String) -> String {
        let mut read_data: Vec<u8> = vec![];
        //        let mut socket;
        let mut stream;
        let mut tls_client;

        let request = self.parse_request(url);

        let request = match request {
            Ok(r) => r,
            Err(e) => return e.to_string(),
        };
        /* TLS setup */
        let config = tls::setup_config();

        let dns_name = webpki::DNSNameRef::try_from_ascii_str(&request.hostname);

        let dns_name = match dns_name {
            Ok(name) => name,
            Err(e) => return e.to_string(),
        };
        tls_client = rustls::ClientSession::new(&Arc::new(config), dns_name);
        let addr = request.for_tcp().to_socket_addrs().unwrap().next().unwrap();
        let mut socket = if let Ok(s) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
            s
        } else {
            return "Failed to connect to socket".to_string();
        };

        stream = rustls::Stream::new(&mut tls_client, &mut socket);

        match stream.write(request.request().as_bytes()) {
            Ok(_) => {}
            Err(e) => todo!(),
        };

        while tls_client.wants_read() {
            tls_client.read_tls(&mut socket).unwrap();
            tls_client.process_new_packets().unwrap();
        }

        tls_client.read_to_end(&mut read_data);
        let content = String::from_utf8_lossy(&read_data);

        content.to_string()
    }
}

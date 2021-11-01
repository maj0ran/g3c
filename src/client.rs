use crate::tls;
use log::info;
use rustls::Session;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

struct GeminiRequest {
    scheme: String,
    address: String,
    port: String,
}

impl GeminiRequest {
    unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(
            (self as *const GeminiRequest) as *const u8,
            std::mem::size_of::<GeminiRequest>(),
        )
    }

    fn for_tcp(&self) -> String {
        format!("{address}:{port}", address = self.address, port = self.port)
    }

    fn for_dns(&self) -> &String {
        &self.address
    }
    fn request(&self) -> String {
        format!(
            "{scheme}://{address}:{port}\r\n",
            scheme = self.scheme,
            address = self.address,
            port = self.port
        )
    }
}

pub struct GeminiClient {
    current_site: String,
}

impl GeminiClient {
    pub fn new() -> Self {
        GeminiClient {
            current_site: String::new(),
        }
    }

    fn parse_request(&self, request: String) -> GeminiRequest {
        let request = request.trim().to_string();

        let scheme = "gemini".to_string();
        let port = "1965".to_string();
        let address = if request.starts_with("gemini://") {
            request["gemini://".len()..].to_string()
        } else {
            request
        };

        info!("scheme: {}", scheme);
        info!("address: {}", address);
        info!("port: {}", port);

        GeminiRequest {
            scheme,
            address,
            port,
        }
    }

    pub fn goto_url(&self, url: String) -> String {
        let mut read_data: Vec<u8> = vec![];
        let mut socket;
        let mut stream;
        let mut tls_client;

        let request = self.parse_request(url);

        /* TLS setup */
        let config = tls::setup_config();

        let dns_name = webpki::DNSNameRef::try_from_ascii_str(&request.address);

        let dns_name = match dns_name {
            Ok(name) => name,
            Err(e) => return e.to_string(),
        };
        tls_client = rustls::ClientSession::new(&Arc::new(config), dns_name);

        socket = TcpStream::connect(request.for_tcp());
        let mut socket = match socket {
            Ok(s) => s,
            Err(e) => return e.to_string(),
        };

        stream = rustls::Stream::new(&mut tls_client, &mut socket);

        match stream.write(request.request().as_bytes()) {
            Ok(_) => {}
            Err(_) => todo!(),
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

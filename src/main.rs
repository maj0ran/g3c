use std::net::TcpStream;
use url::{Url, Host};
use std::io::{Read, Write};
use std::io;
use std::sync::Arc;
use log::{info};
use rustls::Session;
mod tls;

// gemini://breadpunk.club/

fn main ()
{
    let prompt = ">";

    env_logger::init();

    loop {
        let mut cmd = String::new();

        print!("{} ", prompt);

        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut cmd).unwrap();

        match cmd.trim() {
            "q" => break,
            "v" => goto_url(),
            _ => print!("unknown command"),
        }
    }
}

/* if the url-string has no scheme, prepend the scheme gemini://.
 * Always append the port 1965 to the string */
fn make_request (destination: &String) -> String
{
    let mut request = String::new();
    if !destination.starts_with("gemini://") {
        request = format!("gemini://{}", destination);
    };
    request.push_str("/\r\n");
    return request;
}

fn goto_url ()
{
    let mut input = String::new(); // user input
    let mut read_data: Vec<u8> = vec![];
    let mut socket: TcpStream;
    let mut stream;
    let mut tls_client;

    let mut cfg = rustls::ClientConfig::new();
    print!("URL: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let hostname = input.trim();
    let destination = format!("{}:1965", hostname);
    let request = make_request(&hostname.to_string());

    info!("hostname: {}", hostname);
    info!("destination: {}", destination);
    info!("request: {}", request);

    println!("connecting to: {}...", destination);

    /* TLS setup */
    let config = tls::setup_config();
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(&hostname).unwrap();
    tls_client = rustls::ClientSession::new(&Arc::new(config), dns_name);
    socket = TcpStream::connect(&destination).unwrap();
    stream = rustls::Stream::new(&mut tls_client, &mut socket);

    stream.write(request.as_bytes()).unwrap();

    while tls_client.wants_read() {
        tls_client.read_tls(&mut socket).unwrap();
        tls_client.process_new_packets().unwrap();
    }
    tls_client.read_to_end(&mut read_data);
    let content = String::from_utf8_lossy(&read_data);
    println!("{}", content);
}

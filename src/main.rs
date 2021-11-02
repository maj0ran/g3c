
mod interface;
mod client;
mod tls;

use crate::client::GeminiClient;
use interface::Interface;


// gemini://breadpunk.club/
fn main() {
    env_logger::init();

    let client = GeminiClient::new();
    let mut interface = Interface::new(client);
//    client.goto_url("gemini.conman.org/dsfsd".to_string());
    interface.run();
}

/* For the sake of simplicity and testing, implement a TLS-verifier
 * that will just accept every TLS. After we get our thing running,
 * we should implement a more serious TLS-handling */
use std::sync::Arc;
use rustls::{RootCertStore, Certificate, ServerCertVerified, TLSError, ServerCertVerifier};
use webpki::{DNSNameRef};

struct DummyVerifier { }

impl DummyVerifier {
    fn new () -> Self
    {
        DummyVerifier { }
    }
}

impl ServerCertVerifier for DummyVerifier {
    fn verify_server_cert (&self,
                           _: &RootCertStore,
                           _: &[Certificate],
                           _: DNSNameRef,
                           _: &[u8]
    ) -> Result<ServerCertVerified, TLSError>
    {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn setup_config () -> rustls::ClientConfig
{
    let mut cfg = rustls::ClientConfig::new();
    let mut config = rustls::DangerousClientConfig {cfg: &mut cfg};
    let dummy_verifier = Arc::new(DummyVerifier::new());

    config.set_certificate_verifier(dummy_verifier);

    cfg
}

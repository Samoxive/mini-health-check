use std::{
    env::args,
    process::exit,
    sync::Arc,
    time::{Duration, SystemTime},
};

use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, ServerName,
};
use ureq::Agent;

struct Verifier;

impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

// taken from ureq, licensed MIT
fn root_certs() -> rustls::RootCertStore {
    let mut root_cert_store = rustls::RootCertStore::empty();

    let certs = rustls_native_certs::load_native_certs().unwrap_or_default();
    for cert in certs {
        let cert = rustls::Certificate(cert.0);
        // Continue on parsing errors, as native stores often include ancient or syntactically
        // invalid certificates, like root certificates without any X509 extensions.
        // Inspiration: https://github.com/rustls/rustls/blob/633bf4ba9d9521a95f68766d04c22e2b01e68318/rustls/src/anchors.rs#L105-L112
        let _ = root_cert_store.add(&cert);
    }
    root_cert_store
}

fn check_health(agent: Agent, url: &str) -> bool {
    let response = match agent.get(url).call() {
        Ok(response) => response,
        Err(e) => {
            eprintln!("failed to send request to `{}`: {}", url, e);
            exit(1);
        }
    };

    response.status() >= 200 && response.status() < 300
}

fn main() {
    let mut command_args = args();
    let self_path = match command_args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("mini-health-check: no program name\n");
            exit(1);
        }
    };

    let mut tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs())
        .with_no_client_auth();

    tls_config
        .dangerous()
        .set_certificate_verifier(Arc::new(Verifier));

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .redirects(0)
        .user_agent("mini-health-check/0.1")
        .tls_config(Arc::new(tls_config))
        .build();

    let mut health_checked = false;
    let healthy = command_args.all(|arg| {
        health_checked = true;
        check_health(agent.clone(), &arg)
    });

    if !health_checked {
        eprintln!("{}: no URLs given\n", self_path);
        exit(1);
    }

    if !healthy {
        exit(1);
    }
}

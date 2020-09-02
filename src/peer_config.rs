use std::{sync::Arc, time::Duration};

pub fn new_client_cfg(
    idle_timeout_msec: u64,
    keep_alive_interval_msec: u32,
) -> quinn::ClientConfig {
    let mut cfg = quinn::ClientConfigBuilder::default().build();
    let crypto_cfg =
        Arc::get_mut(&mut cfg.crypto).expect("the crypto config should not be shared yet");
    crypto_cfg
        .dangerous()
        .set_certificate_verifier(SkipServerVerification::new());
    cfg.transport = Arc::new(new_transport_cfg(
        idle_timeout_msec,
        keep_alive_interval_msec,
    ));
    cfg
}

pub fn new_our_cfg(
    idle_timeout_msec: u64,
    keep_alive_interval_msec: u32,
    our_cert: quinn::Certificate,
    our_key: quinn::PrivateKey,
) -> Result<quinn::ServerConfig, rustls::TLSError> {
    let mut our_cfg_builder = {
        let mut our_cfg = quinn::ServerConfig::default();
        our_cfg.transport = Arc::new(new_transport_cfg(
            idle_timeout_msec,
            keep_alive_interval_msec,
        ));

        quinn::ServerConfigBuilder::new(our_cfg)
    };
    let _ = our_cfg_builder
        .certificate(quinn::CertificateChain::from_certs(vec![our_cert]), our_key)?
        .use_stateless_retry(true);

    Ok(our_cfg_builder.build())
}

fn new_transport_cfg(
    idle_timeout_msec: u64,
    keep_alive_interval_msec: u32,
) -> quinn::TransportConfig {
    let mut transport_config = quinn::TransportConfig::default();
    let _ = transport_config
        .max_idle_timeout(Some(Duration::from_millis(idle_timeout_msec)))
        .unwrap_or(&mut Default::default());
    let _ = transport_config
        .keep_alive_interval(Some(Duration::from_millis(keep_alive_interval_msec.into())));
    transport_config
}

/// Dummy certificate verifier that treats any certificate as valid.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> std::result::Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

//! TLS helpers for tunnelr — certificate generation, server/client config,
//! and async accept/connect wrappers.
//!
//! For development, [`generate_self_signed_cert`] creates a self-signed
//! certificate via `rcgen`. Production deployments should use real
//! certificates (e.g. via Let's Encrypt / ACME).

use std::io::BufReader;
use std::sync::Arc;

use anyhow::{Context, Result};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::ring as ring_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error as TlsError, SignatureScheme};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{
    client::TlsStream as ClientTlsStream, server::TlsStream as ServerTlsStream, TlsAcceptor,
    TlsConnector,
};
use tracing::{debug, info, instrument};

/// A self-signed certificate and its private key in DER format.
pub struct SelfSignedCert {
    /// DER-encoded X.509 certificate.
    pub cert_der: CertificateDer<'static>,
    /// DER-encoded PKCS#8 private key.
    pub key_der: PrivateKeyDer<'static>,
}

/// Generate a self-signed TLS certificate for development use.
///
/// The certificate includes `localhost` and `127.0.0.1` as Subject
/// Alternative Names, making it usable for local testing.
#[instrument]
pub fn generate_self_signed_cert() -> Result<SelfSignedCert> {
    let key_pair = rcgen::KeyPair::generate().context("failed to generate key pair")?;

    let mut params =
        rcgen::CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .context("failed to create certificate params")?;
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "tunnelr dev");

    let cert = params
        .self_signed(&key_pair)
        .context("failed to self-sign certificate")?;

    debug!("generated self-signed certificate for localhost");

    Ok(SelfSignedCert {
        cert_der: cert.der().clone(),
        key_der: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der())),
    })
}

/// Build a [`rustls::ServerConfig`] from a certificate chain and private key.
///
/// Uses safe default protocol versions (TLS 1.2 and 1.3).
pub fn server_config(
    cert_chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<Arc<rustls::ServerConfig>> {
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(ring_provider::default_provider()))
        .with_safe_default_protocol_versions()
        .context("failed to set protocol versions")?
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .context("failed to build server TLS config")?;
    Ok(Arc::new(config))
}

/// Build a [`rustls::ClientConfig`] that trusts the given server certificate.
///
/// This is intended for development where the server uses a self-signed cert.
/// The client adds the cert to a custom root store rather than relying on
/// system roots.
pub fn client_config_with_cert(
    server_cert: CertificateDer<'static>,
) -> Result<Arc<rustls::ClientConfig>> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store
        .add(server_cert)
        .context("failed to add server certificate to root store")?;
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(ring_provider::default_provider()))
        .with_safe_default_protocol_versions()
        .context("failed to set protocol versions")?
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Accept an incoming TLS connection (server side).
///
/// Wraps the raw TCP stream in a TLS layer using the provided server config.
#[instrument(skip_all)]
pub async fn tls_accept<IO: AsyncRead + AsyncWrite + Unpin>(
    config: Arc<rustls::ServerConfig>,
    stream: IO,
) -> Result<ServerTlsStream<IO>> {
    let acceptor = TlsAcceptor::from(config);
    let tls_stream = acceptor.accept(stream).await.context("TLS accept failed")?;
    debug!("TLS handshake complete (server)");
    Ok(tls_stream)
}

/// Establish an outgoing TLS connection (client side).
///
/// Wraps the raw TCP stream in a TLS layer and validates the server's
/// certificate against the provided client config.
#[instrument(skip_all, fields(%domain))]
pub async fn tls_connect<IO: AsyncRead + AsyncWrite + Unpin>(
    config: Arc<rustls::ClientConfig>,
    domain: &str,
    stream: IO,
) -> Result<ClientTlsStream<IO>> {
    let connector = TlsConnector::from(config);
    let server_name: ServerName<'static> = domain
        .to_string()
        .try_into()
        .context("invalid server name")?;
    let tls_stream = connector
        .connect(server_name, stream)
        .await
        .context("TLS connect failed")?;
    debug!("TLS handshake complete (client)");
    Ok(tls_stream)
}

/// Load a PEM certificate chain and private key from files on disk.
pub fn load_certs_from_pem(
    cert_path: &str,
    key_path: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_file = std::fs::File::open(cert_path)
        .with_context(|| format!("failed to open cert file: {cert_path}"))?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("failed to parse PEM certificates")?;
    if certs.is_empty() {
        anyhow::bail!("no certificates found in {cert_path}");
    }

    let key_file = std::fs::File::open(key_path)
        .with_context(|| format!("failed to open key file: {key_path}"))?;
    let mut key_reader = BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)
        .context("failed to parse PEM private key")?
        .with_context(|| format!("no private key found in {key_path}"))?;

    info!(certs = certs.len(), "loaded TLS certificate chain from PEM files");
    Ok((certs, key))
}

/// TLS client configuration options.
pub struct ClientTlsOptions {
    /// Whether to verify server certificates (default: true).
    pub verify: bool,
    /// Optional path to a custom CA PEM file.
    pub ca_path: Option<String>,
    /// Server hostname for SNI.
    pub hostname: String,
}

/// Build a [`rustls::ClientConfig`] based on the provided options.
///
/// - If `verify` is true and `ca_path` is set, uses that CA file as the trust root.
/// - If `verify` is true and `ca_path` is None, uses the webpki (Mozilla) root store.
/// - If `verify` is false, uses `NoVerifier` (INSECURE — for self-signed dev certs only).
pub fn client_config_from_options(opts: &ClientTlsOptions) -> Result<Arc<rustls::ClientConfig>> {
    let provider = Arc::new(ring_provider::default_provider());

    if !opts.verify {
        debug!("TLS verification DISABLED — using NoVerifier (insecure)");
        let config = rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .context("failed to set protocol versions")?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();
        return Ok(Arc::new(config));
    }

    let mut root_store = rustls::RootCertStore::empty();

    if let Some(ca_path) = &opts.ca_path {
        let ca_file = std::fs::File::open(ca_path)
            .with_context(|| format!("failed to open CA file: {ca_path}"))?;
        let mut reader = BufReader::new(ca_file);
        let ca_certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to parse CA PEM file")?;
        for cert in ca_certs {
            root_store.add(cert).context("failed to add CA certificate")?;
        }
        debug!(ca = ca_path, "using custom CA for TLS verification");
    } else {
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        debug!("using system/webpki root CAs for TLS verification");
    }

    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .context("failed to set protocol versions")?
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Certificate verifier that accepts any server certificate (INSECURE).
///
/// Only for use with self-signed certs in development/self-hosted scenarios
/// when the user explicitly opts in via `--tls-verify=false`.
#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        ring_provider::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};

    #[test]
    fn self_signed_cert_generation() {
        let cert = generate_self_signed_cert().unwrap();
        // Cert DER should be non-empty
        assert!(!cert.cert_der.is_empty());
    }

    #[test]
    fn server_config_from_self_signed() {
        let cert = generate_self_signed_cert().unwrap();
        let config = server_config(vec![cert.cert_der], cert.key_der).unwrap();
        assert!(config.alpn_protocols.is_empty()); // no ALPN by default
    }

    #[test]
    fn client_config_from_self_signed() {
        let cert = generate_self_signed_cert().unwrap();
        let _config = client_config_with_cert(cert.cert_der).unwrap();
    }

    #[tokio::test]
    async fn tls_roundtrip() {
        let cert = generate_self_signed_cert().unwrap();
        let srv_config = server_config(vec![cert.cert_der.clone()], cert.key_der).unwrap();
        let cli_config = client_config_with_cert(cert.cert_der).unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn({
            let srv_config = srv_config.clone();
            async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut tls = tls_accept(srv_config, stream).await.unwrap();
                tokio::io::AsyncWriteExt::write_all(&mut tls, b"hello from server")
                    .await
                    .unwrap();
            }
        });

        let client = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut tls = tls_connect(cli_config, "localhost", stream).await.unwrap();
            let mut buf = vec![0u8; 64];
            let n = tokio::io::AsyncReadExt::read(&mut tls, &mut buf)
                .await
                .unwrap();
            assert_eq!(&buf[..n], b"hello from server");
        });

        server.await.unwrap();
        client.await.unwrap();
    }
}

// Args to configure openssl

use clap::{Args, ValueEnum};
use openssl::{
    error::ErrorStack,
    ssl::{SslConnector, SslConnectorBuilder},
};

#[derive(Args, Debug)]
pub struct OpensslArgs {
    /// The URL to send the request to.
    /// Example: https://localhost:8080
    #[arg(short, long, required = true)]
    pub url: String,
    /// Key file path
    #[arg(short, long, requires = "cert_file")]
    pub key_file: Option<String>,
    /// Certificate file path
    #[arg(long)]
    pub cert_file: Option<String>,
    /// CA file path
    #[arg(long)]
    pub ca_file: Option<String>,
    /// Domain name
    #[arg(short, long, required = true)]
    pub domain: String,
    /// SSL verification mode
    #[arg(long, value_enum, default_value = "peer")]
    pub verify_mode: SslVerifyMode,

    /// SSL/TLS version to use
    #[arg(long, value_enum, default_value = "tls12")]
    pub ssl_min_version: SslVersion,
}

impl OpensslArgs {
    pub fn make_channel(&self) -> Result<tonic::transport::Channel, crate::Error> {
        let endpoint = tonic::transport::Endpoint::from_shared(self.url.clone())?;
        let ch = endpoint.connect_with_connector_lazy(tonic_tls::openssl::TlsConnector::new(
            &endpoint,
            self.make_connector()?.build(),
            self.domain.clone(),
        ));
        Ok(ch)
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SslVerifyMode {
    /// Verify peer certificate
    Peer,
    /// No verification
    None,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SslVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3
    Tls13,
}

impl OpensslArgs {
    pub fn make_connector(&self) -> Result<SslConnectorBuilder, ErrorStack> {
        let mut builder = SslConnector::builder(openssl::ssl::SslMethod::tls_client())?;

        if let Some(key_file) = &self.key_file {
            builder.set_private_key_file(key_file, openssl::ssl::SslFiletype::PEM)?;
        }
        if let Some(cert_file) = &self.cert_file {
            builder.set_certificate_file(cert_file, openssl::ssl::SslFiletype::PEM)?;
        }
        if let Some(ca_file) = &self.ca_file {
            builder.set_ca_file(ca_file)?;
        }

        let min_version = match self.ssl_min_version {
            SslVersion::Tls12 => openssl::ssl::SslVersion::TLS1_2,
            SslVersion::Tls13 => openssl::ssl::SslVersion::TLS1_3,
        };
        builder.set_min_proto_version(Some(min_version))?;

        let verify_mode = match self.verify_mode {
            SslVerifyMode::Peer => openssl::ssl::SslVerifyMode::PEER,
            SslVerifyMode::None => openssl::ssl::SslVerifyMode::NONE,
        };

        builder.set_verify_callback(verify_mode, |ok, x509_ctx| {
            if !ok {
                tracing::debug!(error = %x509_ctx.error(), "Certificate verification failed");
            }
            ok
        });

        // Enable ALPN for HTTP/2
        builder.set_alpn_protos(tonic_tls::openssl::ALPN_H2_WIRE)?;

        Ok(builder)
    }
}

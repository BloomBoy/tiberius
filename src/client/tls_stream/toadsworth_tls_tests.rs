use super::*;

#[test]
fn toad90_explicit_ca_has_no_system_roots() {
    let roots = exclusive_ca(Path::new("docker/certs/customCA.crt")).unwrap();
    assert_eq!(roots.len(), 1);
}

#[test]
fn toad90_missing_ca_keeps_the_error() {
    assert_eq!(exclusive_ca(Path::new("no-such-toad90-ca.pem")).unwrap_err(), Error::Io {
        kind: IoErrorKind::InvalidData,
        message: "Could not read provided CA certificate!".into(),
    });
}

#[test]
fn toad90_certificate_errors_keep_the_adapter_classifier() {
    for cert in [CertificateError::UnknownIssuer, CertificateError::NotValidForName,
                 CertificateError::Expired, CertificateError::NotValidYet] {
        let expected = format!("invalid peer certificate: {cert:?}");
        let error = handshake_error(io::Error::new(IoErrorKind::InvalidData, RustlsError::InvalidCertificate(cert)));
        assert_eq!(error, Error::Io { kind: IoErrorKind::InvalidData, message: expected });
    }
}

#[test]
fn toad90_non_certificate_errors_keep_their_text() {
    assert_eq!(handshake_error(io::Error::new(IoErrorKind::ConnectionReset, "reset by peer")), Error::Io {
        kind: IoErrorKind::ConnectionReset, message: "reset by peer".into(),
    });
}

#[cfg(feature = "toadsworth-compat")]
#[test]
fn toad90_connect_error_preserves_the_exact_io_prefix() {
    let error = handshake_error(io::Error::new(IoErrorKind::InvalidData,
        RustlsError::InvalidCertificate(CertificateError::UnknownIssuer)));
    assert_eq!(error.to_string(), "An error occured during the attempt of performing I/O: invalid peer certificate: UnknownIssuer");
}

#[cfg(feature = "toadsworth-compat")]
#[test]
fn toad90_config_uses_only_the_last_named_ca() {
    let mut config = Config::new();
    config.trust_cert_ca("no-such-ca.pem");
    config.trust_cert_ca("docker/certs/customCA.crt");
    assert_eq!(build_trust_store(&config.trust).unwrap().len(), 1);
}

// rustls 0.21 named an unsupported or mismatched signature algorithm
// BadSignature and a missing serverAuth EKU Other(RequiredEkuNotFound); 0.23
// reports context variants instead (TOAD-90, Fable P2).
#[test]
#[allow(deprecated)]
fn toad90_signature_and_purpose_errors_keep_the_shipped_names() {
    use tokio_rustls::rustls::ExtendedKeyPurpose;
    let cases = [
        (CertificateError::UnsupportedSignatureAlgorithm, "BadSignature"),
        (CertificateError::UnsupportedSignatureAlgorithmContext {
            signature_algorithm_id: vec![6, 9, 42, 134, 72, 134, 247, 13, 1, 1, 5],
            supported_algorithms: Vec::new(),
        }, "BadSignature"),
        (CertificateError::UnsupportedSignatureAlgorithmForPublicKeyContext {
            signature_algorithm_id: vec![6, 8, 42],
            public_key_algorithm_id: vec![6, 7, 42],
        }, "BadSignature"),
        (CertificateError::InvalidPurpose, "Other(RequiredEkuNotFound)"),
        (CertificateError::InvalidPurposeContext {
            required: ExtendedKeyPurpose::ServerAuth,
            presented: vec![ExtendedKeyPurpose::ClientAuth],
        }, "Other(RequiredEkuNotFound)"),
    ];
    for (cert, word) in cases {
        let error = handshake_error(io::Error::new(IoErrorKind::InvalidData, RustlsError::InvalidCertificate(cert)));
        assert_eq!(error, Error::Io {
            kind: IoErrorKind::InvalidData,
            message: format!("invalid peer certificate: {word}"),
        });
    }
}

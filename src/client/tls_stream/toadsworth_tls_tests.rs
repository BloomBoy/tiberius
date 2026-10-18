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

// The same two refusals as produced by rustls' own verifier for real
// certificates, not constructed error values: a SHA-1 signed server
// certificate and one whose only EKU is clientAuth (docker/certs/toad90, valid
// 2026-10-16 to 2046; verified at 2027-01-15).
#[test]
fn toad90_real_verifier_refusals_keep_the_shipped_names() {
    use tokio_rustls::rustls::client::danger::ServerCertVerifier;
    use tokio_rustls::rustls::crypto::aws_lc_rs;
    use tokio_rustls::rustls::client::WebPkiServerVerifier;
    use tokio_rustls::rustls::pki_types::{ServerName, UnixTime};
    let first = |path: &str| CertificateDer::pem_slice_iter(&fs::read(path).unwrap()).next().unwrap().unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(first("docker/certs/toad90/ca.pem")).unwrap();
    let verifier = WebPkiServerVerifier::builder_with_provider(
        std::sync::Arc::new(roots),
        std::sync::Arc::new(aws_lc_rs::default_provider()),
    )
    .build()
    .unwrap();
    let at = UnixTime::since_unix_epoch(std::time::Duration::from_secs(1_800_000_000));
    for (leaf, word) in [
        ("docker/certs/toad90/server-sha1.pem", "BadSignature"),
        ("docker/certs/toad90/server-client-eku.pem", "Other(RequiredEkuNotFound)"),
    ] {
        let refused = verifier
            .verify_server_cert(&first(leaf), &[], &ServerName::try_from("localhost").unwrap(), &[], at)
            .expect_err(leaf);
        let error = handshake_error(io::Error::new(IoErrorKind::InvalidData, refused));
        assert_eq!(error, Error::Io {
            kind: IoErrorKind::InvalidData,
            message: format!("invalid peer certificate: {word}"),
        }, "{leaf}");
    }
}

// A CA can arrive as PEM text instead of a path (an organisation register
// entry). It follows the shipped policy of a named CA file: it replaces the
// system roots and any earlier CA, holds exactly one certificate, and is
// never echoed in an error (TOAD-90, kort 71).
#[cfg(feature = "toadsworth-compat")]
const TOAD90_BROKEN_PEM: &str = "Could not read provided CA certificate! The PEM text is not a valid certificate.";

#[cfg(feature = "toadsworth-compat")]
#[test]
fn toad90_pem_text_ca_has_no_system_roots() {
    let mut config = Config::new();
    config.trust_cert_ca_pem(fs::read("docker/certs/customCA.crt").unwrap());
    assert_eq!(build_trust_store(&config.trust).unwrap().len(), 1);
}

#[cfg(feature = "toadsworth-compat")]
#[test]
fn toad90_pem_text_and_a_path_replace_each_other() {
    let mut config = Config::new();
    config.trust_cert_ca("no-such-ca.pem");
    config.trust_cert_ca_pem(fs::read("docker/certs/customCA.crt").unwrap());
    assert_eq!(build_trust_store(&config.trust).unwrap().len(), 1);
    let mut config = Config::new();
    config.trust_cert_ca_pem(b"-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n".to_vec());
    config.trust_cert_ca("docker/certs/customCA.crt");
    assert_eq!(build_trust_store(&config.trust).unwrap().len(), 1);
}

#[cfg(feature = "toadsworth-compat")]
#[test]
fn toad90_broken_pem_text_is_refused_without_echoing_it() {
    for pem in [
        "-----BEGIN CERTIFICATE-----\nnot*base64*secret\n-----END CERTIFICATE-----\n",
        "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n",
        "-----BEGIN CERTIFICATE-----\nMIIB\n",
    ] {
        let mut config = Config::new();
        config.trust_cert_ca_pem(pem.as_bytes().to_vec());
        let error = build_trust_store(&config.trust).unwrap_err();
        assert_eq!(error, Error::Io { kind: IoErrorKind::InvalidData, message: TOAD90_BROKEN_PEM.into() }, "{pem}");
    }
}

#[cfg(feature = "toadsworth-compat")]
#[test]
fn toad90_pem_text_must_hold_exactly_one_certificate() {
    for pem in [
        fs::read("docker/certs/server-full.crt").unwrap(),
        b"-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n".to_vec(),
    ] {
        let mut config = Config::new();
        config.trust_cert_ca_pem(pem);
        let error = build_trust_store(&config.trust).unwrap_err();
        assert_eq!(error, Error::Io {
            kind: IoErrorKind::InvalidInput,
            message: "Provided CA certificate PEM text contain 0 or more than 1 certs".into(),
        });
    }
}

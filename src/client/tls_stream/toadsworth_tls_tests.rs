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

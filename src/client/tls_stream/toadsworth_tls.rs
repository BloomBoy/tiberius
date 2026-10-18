//! Compatibility with the shipped fork: an explicit CA replaces system roots,
//! accepts exactly one certificate, and keeps the adapter's error vocabulary.
use super::*;
use tokio_rustls::rustls::CertificateError;

pub(super) fn exclusive_ca(path: &Path) -> crate::Result<RootCertStore> {
    let bytes = fs::read(path).map_err(|_| Error::Io {
        kind: IoErrorKind::InvalidData,
        message: "Could not read provided CA certificate!".into(),
    })?;
    let cert = match path.extension() {
        Some(ext) if ext.eq_ignore_ascii_case("pem") || ext.eq_ignore_ascii_case("crt") => {
            let certs = CertificateDer::pem_slice_iter(&bytes)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Io { kind: IoErrorKind::InvalidData, message: e.to_string() })?;
            if certs.len() != 1 {
                return Err(Error::Io {
                    kind: IoErrorKind::InvalidInput,
                    message: format!("Certificate file {} contain 0 or more than 1 certs", path.to_string_lossy()),
                });
            }
            certs.into_iter().next().expect("exactly one certificate")
        }
        Some(ext) if ext.eq_ignore_ascii_case("der") => CertificateDer::from(bytes),
        _ => return Err(Error::Io {
            kind: IoErrorKind::InvalidInput,
            message: "Provided CA certificate with unsupported file-extension! Supported types are pem, crt and der.".into(),
        }),
    };
    let mut store = RootCertStore::empty();
    store.add(cert)?;
    Ok(store)
}

/// A CA given as PEM text, under the same policy as [`exclusive_ca`]. The text
/// may be key material pasted by mistake, so no error repeats any of it.
pub(super) fn exclusive_ca_pem(pem: &[u8]) -> crate::Result<RootCertStore> {
    let unreadable = || Error::Io {
        kind: IoErrorKind::InvalidData,
        message: "Could not read provided CA certificate! The PEM text is not a valid certificate.".into(),
    };
    let certs = CertificateDer::pem_slice_iter(pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| unreadable())?;
    if certs.len() != 1 {
        return Err(Error::Io {
            kind: IoErrorKind::InvalidInput,
            message: "Provided CA certificate PEM text contain 0 or more than 1 certs".into(),
        });
    }
    let mut store = RootCertStore::empty();
    store
        .add(certs.into_iter().next().expect("exactly one certificate"))
        .map_err(|_| unreadable())?;
    Ok(store)
}

pub(super) fn handshake_error(err: io::Error) -> Error {
    if let Some(RustlsError::InvalidCertificate(cert)) =
        err.get_ref().and_then(|inner| inner.downcast_ref::<RustlsError>())
    {
        // rustls 0.23 adds context to these variants. Keep the established
        // classifier names so the application presents the same guidance.
        #[allow(deprecated)]
        let word = match cert {
            CertificateError::NotValidForNameContext { .. } => "NotValidForName".into(),
            // rustls 0.21 mapped every unsupported or mismatched signature
            // algorithm to BadSignature, and a missing serverAuth EKU fell
            // through to webpki's RequiredEkuNotFound.
            CertificateError::UnsupportedSignatureAlgorithm
            | CertificateError::UnsupportedSignatureAlgorithmContext { .. }
            | CertificateError::UnsupportedSignatureAlgorithmForPublicKeyContext { .. } => {
                "BadSignature".into()
            }
            CertificateError::InvalidPurpose | CertificateError::InvalidPurposeContext { .. } => {
                "Other(RequiredEkuNotFound)".into()
            }
            CertificateError::ExpiredContext { .. } => "Expired".into(),
            CertificateError::NotValidYetContext { .. } => "NotValidYet".into(),
            CertificateError::Other(other) => format!("Other({:?})", other.0),
            other => format!("{other:?}"),
        };
        return Error::Io {
            kind: err.kind(),
            message: format!("invalid peer certificate: {word}"),
        };
    }
    Error::from(err)
}

#[cfg(test)]
#[path = "toadsworth_tls_tests.rs"]
mod tests;

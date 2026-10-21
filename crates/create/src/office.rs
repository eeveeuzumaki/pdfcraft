//! Local Office document conversion using a user's installed LibreOffice.
//! No document is sent to an online service. Supported: DOCX/XLSX/PPTX/ODT/ODS/ODP.
//! Conversion runs in an isolated temporary profile so a running LibreOffice
//! desktop instance does not steal our conversion request.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::CreateError;

const MAX_INPUT: usize = 128 * 1024 * 1024;
const MAX_OUTPUT: u64 = 256 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(90);

/// Return the extension of an eligible local Office file, or None. Do not
/// accept macro-enabled formats (.docm, .xlsm, .pptm).
pub fn format(name: &str) -> Option<&'static str> {
    let extension = name.rsplit('.').next()?.to_ascii_lowercase();
    match extension.as_str() {
        "docx" => Some("docx"),
        "xlsx" => Some("xlsx"),
        "pptx" => Some("pptx"),
        "odt" => Some("odt"),
        "ods" => Some("ods"),
        "odp" => Some("odp"),
        _ => None,
    }
}

/// OOXML and ODF formats are ZIP containers, never plain text or a PDF header.
/// This is only a quick format check; LibreOffice does the full validation.
pub fn looks_like_office(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
}

/// Convert one Office document entirely on-device. The input is copied to a
/// private temp directory under a fixed, extension-only name; it is not given
/// to a shell or remote API. The returned PDF is checked by the engine parser.
pub fn convert(name: &str, bytes: &[u8]) -> Result<Vec<u8>, CreateError> {
    let ext = format(name).ok_or_else(|| CreateError::Invalid("unsupported Office format (use DOCX, XLSX, PPTX, ODT, ODS or ODP)".into()))?;
    if !looks_like_office(bytes) || bytes.len() > MAX_INPUT {
        return Err(CreateError::Invalid("Office document has an invalid header or exceeds 128 MiB".into()));
    }
    let temp = tempfile::tempdir().map_err(|e| CreateError::Invalid(format!("couldn't create temporary Office workspace: {e}")))?;
    let profile = temp.path().join("lo-profile");
    std::fs::create_dir(&profile).map_err(|e| CreateError::Invalid(format!("couldn't prepare LibreOffice profile: {e}")))?;
    let url = url::Url::from_directory_path(&profile)
        .map_err(|()| CreateError::Invalid("couldn't construct a private LibreOffice profile URL".into()))?;
    let input = temp.path().join(format!("source.{ext}"));
    std::fs::write(&input, bytes).map_err(|e| CreateError::Invalid(format!("couldn't stage the Office file: {e}")))?;
    let output = temp.path().join("source.pdf");

    let candidates = candidates();
    let mut launched = false;
    for executable in candidates {
        let mut command = Command::new(&executable);
        command
            .arg(format!("-env:UserInstallation={url}"))
            .arg("--headless")
            .arg("--convert-to")
            .arg("pdf")
            .arg("--outdir")
            .arg(temp.path())
            .arg(&input)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(CreateError::Invalid(format!("couldn't launch LibreOffice: {e}"))),
        };
        launched = true;
        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(_)) => return Err(CreateError::Invalid("LibreOffice could not convert this document to PDF".into())),
                Ok(None) if start.elapsed() < TIMEOUT => std::thread::sleep(Duration::from_millis(100)),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(CreateError::Invalid("LibreOffice conversion timed out after 90 seconds".into()));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(CreateError::Invalid(format!("couldn't finish LibreOffice conversion: {e}")));
                }
            }
        }
        break;
    }
    if !launched {
        return Err(CreateError::Invalid(
            "LibreOffice is required for offline Office conversion. Install it on this computer and retry.".into(),
        ));
    }
    let metadata = std::fs::metadata(&output).map_err(|_| CreateError::Invalid("LibreOffice did not create a PDF".into()))?;
    if metadata.len() < 8 || metadata.len() > MAX_OUTPUT {
        return Err(CreateError::Invalid("converted PDF is empty or exceeds 256 MiB".into()));
    }
    let pdf = std::fs::read(output).map_err(|e| CreateError::Invalid(format!("couldn't read the converted PDF: {e}")))?;
    if !pdf.get(..pdf.len().min(1024)).unwrap_or_default().windows(5).any(|w| w == b"%PDF-") {
        return Err(CreateError::Invalid("LibreOffice did not produce a valid PDF header".into()));
    }
    Ok(pdf)
}

/// Prefer the executable in PATH; recognize normal installed locations for
/// non-technical users whose graphical installer didn't add it to PATH.
fn candidates() -> Vec<PathBuf> {
    let mut choices = vec![PathBuf::from("soffice"), PathBuf::from("libreoffice")];
    #[cfg(target_os = "macos")]
    choices.push(PathBuf::from("/Applications/LibreOffice.app/Contents/MacOS/soffice"));
    #[cfg(target_os = "windows")]
    {
        choices.push(PathBuf::from(r"C:\Program Files\LibreOffice\program\soffice.exe"));
        choices.push(PathBuf::from(r"C:\Program Files (x86)\LibreOffice\program\soffice.exe"));
    }
    choices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_office_formats_and_macro_safety() {
        for name in ["draft.docx", "sheet.XLSX", "slides.pptx", "text.odt", "table.ods", "deck.odp"] {
            assert!(format(name).is_some(), "{name}");
        }
        for name in ["macro.docm", "macro.xlsm", "macro.pptm", "fake.zip", "note.txt", ".docx"] {
            assert!(format(name).is_none(), "{name}");
        }
        assert!(looks_like_office(b"PK\x03\x04archive"));
        assert!(!looks_like_office(b"%PDF-1.7"));
        assert!(!looks_like_office(b"not a zip"));
    }

    #[test]
    fn malformed_office_files_fail_without_requiring_libreoffice() {
        assert!(convert("report.docx", b"not actually a ZIP").is_err());
        assert!(convert("report.docm", b"PK\x03\x04some bytes").is_err());
    }
}

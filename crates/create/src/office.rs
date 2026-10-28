//! Local Office document conversion using a user's installed LibreOffice.
//! No document is sent to an online service. Supported: DOCX/XLSX/PPTX/ODT/ODS/ODP.
//! Conversion runs in an isolated temporary profile so a running LibreOffice
//! desktop instance does not steal our conversion request.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::CreateError;

const MAX_INPUT: usize = 128 * 1024 * 1024;
const MAX_OUTPUT: u64 = 256 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(90);

/// Return the extension of an eligible local Office file, or None. Do not
/// accept macro-enabled formats (.docm, .xlsm, .pptm).
pub fn format(name: &str) -> Option<&'static str> {
    let (stem, extension) = name.rsplit_once('.')?;
    if stem.is_empty() {
        return None;
    }
    let extension = extension.to_ascii_lowercase();
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
/// Require a local-file header and a complete ZIP end-of-central-directory
/// record, including its declared comment, before launching LibreOffice.
/// This is only a quick check; LibreOffice performs full format validation.
pub fn looks_like_office(bytes: &[u8]) -> bool {
    if bytes.len() < 52 || !bytes.starts_with(b"PK\x03\x04") {
        return false;
    }
    // ZIP comments can be up to 65,535 bytes; the EOCD record itself is 22.
    let start = bytes.len().saturating_sub(65_557);
    (start..=bytes.len() - 22).rev().any(|pos| {
        if bytes.get(pos..pos + 4) != Some(b"PK\x05\x06".as_slice()) {
            return false;
        }
        // Office ZIPs are single-disk archives. Reject split archives and
        // inconsistent per-disk entry counts before invoking LibreOffice.
        let field = |offset| u16::from_le_bytes([bytes[pos + offset], bytes[pos + offset + 1]]);
        if field(4) != 0 || field(6) != 0 || field(8) != field(10) {
            return false;
        }
        // ZIP64 uses sentinel values here; leave those archives to LibreOffice.
        // For ordinary ZIPs, the central directory must end before the EOCD.
        let field32 = |offset| bytes.get(pos + offset..pos + offset + 4).and_then(|value| <[u8; 4]>::try_from(value).ok()).map(u32::from_le_bytes);
        let (Some(size), Some(offset)) = (field32(12), field32(16)) else {
            return false;
        };
        if size != u32::MAX && offset != u32::MAX && u64::from(size) + u64::from(offset) > pos as u64 {
            return false;
        }
        let comment_len = field(20) as usize;
        pos + 22 + comment_len == bytes.len()
    })
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
    let url =
        url::Url::from_directory_path(&profile).map_err(|()| CreateError::Invalid("couldn't construct a private LibreOffice profile URL".into()))?;
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
        return Err(CreateError::Invalid("LibreOffice is required for offline Office conversion. Install it on this computer and retry.".into()));
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
    #[allow(unused_mut)]
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
        // A plausible ZIP must have an end-of-central-directory record.
        let zipish = [b"PK\x03\x04".as_slice(), &[0; 46], b"PK\x05\x06", &[0; 18]].concat();
        assert!(looks_like_office(&zipish));
        // The complete EOCD record must end at the declared ZIP comment.
        let mut with_comment = zipish.clone();
        let len = with_comment.len();
        with_comment[len - 2..].copy_from_slice(&3u16.to_le_bytes());
        with_comment.extend_from_slice(b"abc");
        assert!(looks_like_office(&with_comment));
        assert!(!looks_like_office(&with_comment[..with_comment.len() - 1]));
        assert!(!looks_like_office(&zipish[..zipish.len() - 1]));
        let mut trailing_junk = zipish.clone();
        trailing_junk.extend_from_slice(b"junk");
        assert!(!looks_like_office(&trailing_junk));
        let mut missing_comment = zipish.clone();
        let len = missing_comment.len();
        missing_comment[len - 2..].copy_from_slice(&3u16.to_le_bytes());
        assert!(!looks_like_office(&missing_comment));
        assert!(!looks_like_office(b"PK\x03\x04"));
        assert!(!looks_like_office(b"PK\x03\x04archive"));
        assert!(!looks_like_office(b"%PDF-1.7"));
        assert!(!looks_like_office(b"not a zip"));
    }

    #[test]
    fn reject_split_zip_archives_before_invoking_libreoffice() {
        // This lightweight preflight does not validate the central directory.
        // It does require single-disk EOCD metadata with consistent counts.
        let zipish = [b"PK\x03\x04".as_slice(), &[0; 46], b"PK\x05\x06", &[0; 18]].concat();
        let eocd = zipish.len() - 22;
        for field in [4, 6, 8, 10] {
            let mut invalid = zipish.clone();
            invalid[eocd + field..eocd + field + 2].copy_from_slice(&1u16.to_le_bytes());
            assert!(!looks_like_office(&invalid), "invalid EOCD field at {field}");
        }
        let mut single_disk = zipish;
        for field in [8, 10] {
            single_disk[eocd + field..eocd + field + 2].copy_from_slice(&1u16.to_le_bytes());
        }
        assert!(looks_like_office(&single_disk));
    }

    #[test]
    fn reject_impossible_office_zip_central_directory_bounds() {
        // A central directory cannot extend beyond the end-of-directory record.
        let zipish = [b"PK\x03\x04".as_slice(), &[0; 46], b"PK\x05\x06", &[0; 18]].concat();
        let eocd = zipish.len() - 22;
        for (size, offset) in [(eocd as u32 + 1, 0), (0, eocd as u32 + 1), (eocd as u32, 1)] {
            let mut invalid = zipish.clone();
            invalid[eocd + 12..eocd + 16].copy_from_slice(&size.to_le_bytes());
            invalid[eocd + 16..eocd + 20].copy_from_slice(&offset.to_le_bytes());
            assert!(!looks_like_office(&invalid), "size {size}, offset {offset}");
        }
        let mut within_bounds = zipish;
        within_bounds[eocd + 12..eocd + 16].copy_from_slice(&20u32.to_le_bytes());
        within_bounds[eocd + 16..eocd + 20].copy_from_slice(&((eocd - 20) as u32).to_le_bytes());
        assert!(looks_like_office(&within_bounds));
    }

    #[test]
    fn malformed_office_files_fail_without_requiring_libreoffice() {
        assert!(convert("report.docx", b"not actually a ZIP").is_err());
        assert!(convert("report.docm", b"PK\x03\x04some bytes").is_err());
    }
}

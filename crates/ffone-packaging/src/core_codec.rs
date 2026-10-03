use super::*;

pub(super) fn is_flat_recovery_payload(path: &str) -> bool {
    let mut segments = path.split('/');
    matches!(
        (segments.next(), segments.next(), segments.next()),
        (Some("audio"), Some(file), None) if file.ends_with(".ogg")
    ) || matches!(
        (path.split('/').next(), path.split('/').nth(1), path.split('/').nth(2)),
        (Some("models"), Some(file), None) if file.ends_with(".glb")
    )
}

pub(super) fn checked_payload_bytes(files: &[ProjectFile], description: &str) -> Result<u64, String> {
    files.iter().try_fold(0_u64, |total, file| {
        total.checked_add(file.bytes).ok_or_else(|| {
            format!(
                "{description} payload byte count overflow at asset {:?}",
                file.path
            )
        })
    })
}

pub(super) fn copy_verified_payload(
    path: &Path,
    expected: &ProjectFile,
    output: &mut impl Write,
) -> Result<(), String> {
    let metadata = regular_metadata(path)?;
    if metadata.len() != expected.bytes {
        return Err(format!(
            "{} length mismatch: expected {}, found {}",
            path.display(),
            expected.bytes,
            metadata.len()
        ));
    }
    let file =
        fs::File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = blake3::Hasher::new();
    let mut copied = 0_u64;
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        output
            .write_all(&buffer[..count])
            .map_err(|error| format!("cannot write recovery archive payload: {error}"))?;
        hasher.update(&buffer[..count]);
        copied = copied
            .checked_add(
                u64::try_from(count)
                    .map_err(|_| "archive read size cannot be represented as u64".to_owned())?,
            )
            .ok_or_else(|| "archive copied byte count overflow".to_owned())?;
    }
    if copied != expected.bytes {
        return Err(format!(
            "{} changed length while packing: expected {}, copied {copied}",
            path.display(),
            expected.bytes
        ));
    }
    let actual_hash = hasher.finalize().to_hex().to_string();
    if actual_hash != expected.blake3 {
        return Err(format!(
            "{} BLAKE3 mismatch while packing: expected {}, found {actual_hash}",
            path.display(),
            expected.blake3
        ));
    }
    Ok(())
}

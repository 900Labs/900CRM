use std::path::{Path, PathBuf};

const SECRET_DENYLIST: &[&str] = &[
    ".ssh",
    ".aws",
    ".gnupg",
    ".config",
    ".env",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "credentials",
    ".netrc",
    ".npmrc",
    ".gitconfig",
    ".docker",
];

/// Validate a frontend-supplied READ path (import/preflight/preview) and return the
/// lexically-normalized absolute path, or an error if the path is empty, escapes its
/// root via `..`, or names a sensitive (secret/credential) location.
///
/// This is a defense-in-depth mitigation against a compromised webview exfiltrating
/// secrets or overwriting system files via the import/export/backup commands. It does
/// not restrict which user directories may be used (legitimate imports from USB drives,
/// external volumes, and arbitrary user folders remain supported) but blocks known
/// secret/credential locations anywhere on the path. Symlink traversal past these checks
/// remains a documented residual risk; the primary trust boundary is the OS file dialog.
pub fn validate_import_path(raw: &str) -> Result<PathBuf, String> {
    validate(raw)
}

/// Validate a frontend-supplied WRITE path (export/backup) using the same escape and
/// secret-denylist rules as reads. Existing-file overwrite is intentionally permitted to
/// preserve current export behaviour.
pub fn validate_export_path(raw: &str) -> Result<PathBuf, String> {
    validate(raw)
}

fn validate(raw: &str) -> Result<PathBuf, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Path must not be empty".to_string());
    }

    let input = Path::new(raw);
    let absolute = if input.is_absolute() {
        input.to_path_buf()
    } else {
        let home =
            home_dir().ok_or_else(|| "Unable to determine the user home directory".to_string())?;
        home.join(input)
    };

    let normalized = lexical_normalize(&absolute)
        .ok_or_else(|| "Path escapes its root directory and is not allowed".to_string())?;

    reject_if_secret(&normalized)?;
    Ok(normalized)
}

fn lexical_normalize(absolute: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    Some(out)
}

fn reject_if_secret(path: &Path) -> Result<(), String> {
    let lower = path.to_string_lossy().to_lowercase();
    for forbidden in SECRET_DENYLIST {
        if lower.contains(forbidden) {
            return Err(format!(
                "Path refers to a sensitive location and is not allowed: {}",
                path.display()
            ));
        }
    }

    if let Some(name) = path.file_name() {
        let name_lower = name.to_string_lossy().to_lowercase();
        if name_lower.starts_with('.')
            && (name_lower.contains("key")
                || name_lower.contains("cred")
                || name_lower.contains("token")
                || name_lower.contains("secret"))
        {
            return Err(format!(
                "Path refers to a sensitive location and is not allowed: {}",
                path.display()
            ));
        }
    }

    Ok(())
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

use std::path::{Path, PathBuf};

/// Constants
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
pub const MAX_PATH_LENGTH: usize = 4096;

/// Validates that a path is safe for reading
pub fn validate_path_for_read(path: &str) -> Result<PathBuf, String> {
    // Check for empty path
    if path.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    // Check length
    if path.len() > MAX_PATH_LENGTH {
        return Err(format!("Path too long (max {} chars)", MAX_PATH_LENGTH));
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err("Invalid path: contains null bytes".to_string());
    }

    // Check for suspicious patterns
    if path.contains("..") {
        return Err("Path traversal detected: '..' not allowed".to_string());
    }

    let path_buf = PathBuf::from(path);
    Ok(path_buf)
}

/// Validates that a path is safe for writing
pub fn validate_path_for_write(path: &str) -> Result<PathBuf, String> {
    // First do basic validation
    let path_buf = validate_path_for_read(path)?;

    // Additional check: parent directory should exist or be creatable
    if let Some(parent) = path_buf.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            // Parent doesn't exist - check if we can create it
            // For now, just allow it - we'll attempt to create during write
        }
    }

    Ok(path_buf)
}

/// Checks if a path is within a sandbox directory
pub fn is_in_sandbox(path: &Path, sandbox_root: &Path) -> bool {
    // Get canonical paths
    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // If canonicalize fails, try to at least check the prefix
            let path_normalized = normalize_path(path);
            let root_normalized = normalize_path(sandbox_root);
            return path_normalized.starts_with(&root_normalized);
        }
    };

    let canonical_root = match sandbox_root.canonicalize() {
        Ok(r) => r,
        Err(_) => normalize_path(sandbox_root),
    };

    canonical_path.starts_with(&canonical_root)
}

/// Normalizes a path without requiring it to exist
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory
            }
            other => {
                result.push(other);
            }
        }
    }

    result
}

/// Safe directory reading with sandbox validation
pub fn read_dir_safe(root: &Path, sandbox_root: &Path) -> Result<Vec<PathBuf>, String> {
    // Validate sandbox boundaries
    if !is_in_sandbox(root, sandbox_root) {
        return Err("Access denied: Directory outside sandbox".to_string());
    }

    let mut entries = Vec::new();

    let dir_entries =
        std::fs::read_dir(root).map_err(|e| format!("Cannot read directory: {}", e))?;

    for entry_result in dir_entries {
        let entry = entry_result.map_err(|e| format!("Error reading entry: {}", e))?;
        let path = entry.path();

        // Double-check each entry is in sandbox
        if is_in_sandbox(&path, sandbox_root) {
            entries.push(path);
        }
    }

    Ok(entries)
}

/// Safe file reading with size limit
pub fn read_file_safe(path: &Path, max_size: u64) -> Result<String, String> {
    // Check file exists and get metadata
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Cannot access file: {}", e))?;

    // Enforce size limit (prevent DoS)
    if metadata.len() > max_size {
        return Err(format!(
            "File too large: {} MB (max: {} MB)",
            metadata.len() / 1024 / 1024,
            max_size / 1024 / 1024
        ));
    }

    // Verify it's actually a file
    if !metadata.is_file() {
        return Err("Not a regular file".to_string());
    }

    std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Safe file writing with validation
pub fn write_file_safe(path: &Path, content: &str, sandbox_root: &Path) -> Result<(), String> {
    // Validate path is in sandbox
    if !is_in_sandbox(path, sandbox_root) {
        return Err("Access denied: Cannot write outside sandbox".to_string());
    }

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory: {}", e))?;

            // Set secure permissions on new directory
            #[cfg(unix)]
            set_secure_permissions(parent)?;
        }
    }

    // Write file
    std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    // Set secure permissions on file
    #[cfg(unix)]
    set_secure_permissions(path)?;

    Ok(())
}

/// Sets secure file/directory permissions (Unix only)
#[cfg(unix)]
pub fn set_secure_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    // 0o700 = rwx------ (owner only, no group/other access)
    let perms = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(path, perms)
        .map_err(|e| format!("Failed to set permissions: {}", e))
}

/// No-op on non-Unix systems
#[cfg(not(unix))]
pub fn set_secure_permissions(_path: &Path) -> Result<(), String> {
    // Windows: equivalent via ACLs would be needed, skip for now
    Ok(())
}

/// Get the user's home directory as sandbox root
pub fn get_sandbox_root() -> Result<PathBuf, String> {
    directories::UserDirs::new()
        .ok_or_else(|| "Cannot determine home directory".to_string())
        .map(|dirs| dirs.home_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detected() {
        assert!(validate_path_for_read("../../../etc/passwd").is_err());
        assert!(validate_path_for_write("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_null_byte_detected() {
        assert!(validate_path_for_read("valid\0path").is_err());
    }

    #[test]
    fn test_path_length_limit() {
        let long_path = "a".repeat(5000);
        assert!(validate_path_for_read(&long_path).is_err());
    }

    #[test]
    fn test_empty_path_rejected() {
        assert!(validate_path_for_read("").is_err());
        assert!(validate_path_for_read("   ").is_err());
    }

    #[test]
    fn test_valid_path_accepted() {
        assert!(validate_path_for_read("documents/file.txt").is_ok());
        assert!(validate_path_for_write("documents/newfile.txt").is_ok());
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("./documents/../documents/./file.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("documents/file.txt"));
    }
}

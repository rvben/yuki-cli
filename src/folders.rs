use crate::error::YukiError;

/// The archive folders Yuki exposes, as (canonical name, aliases, id).
const FOLDERS: &[(&str, &[&str], i32)] = &[
    ("inkoop", &["purchase"], 1),
    ("verkoop", &["sales"], 2),
    ("bank", &[], 3),
    ("personeel", &["personnel"], 4),
    ("belasting", &["tax"], 5),
    ("uitzoeken", &[], 7),
    ("overig-financieel", &["other"], 8),
];

/// Comma-separated list of accepted folder names, for help text and errors.
pub fn known_folders() -> String {
    FOLDERS
        .iter()
        .map(|(name, _, _)| *name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolve a folder name or numeric ID to a Yuki archive folder ID.
///
/// Accepts a canonical name, a known alias, or a bare integer ID. An unrecognised
/// name is an error rather than a fallback: silently resolving to folder 0 would
/// return the whole archive and read as a legitimate result.
pub fn folder_id(name: &str) -> Result<i32, YukiError> {
    let trimmed = name.trim();
    if let Ok(id) = trimmed.parse::<i32>() {
        return Ok(id);
    }
    let lowered = trimmed.to_ascii_lowercase();
    FOLDERS
        .iter()
        .find(|(canonical, aliases, _)| {
            *canonical == lowered || aliases.contains(&lowered.as_str())
        })
        .map(|(_, _, id)| *id)
        .ok_or_else(|| {
            YukiError::Config(format!(
                "unknown folder: {name} (expected one of: {}, or a numeric folder ID)",
                known_folders()
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_canonical_names() {
        assert_eq!(folder_id("inkoop").unwrap(), 1);
        assert_eq!(folder_id("verkoop").unwrap(), 2);
        assert_eq!(folder_id("bank").unwrap(), 3);
        assert_eq!(folder_id("personeel").unwrap(), 4);
        assert_eq!(folder_id("belasting").unwrap(), 5);
        assert_eq!(folder_id("uitzoeken").unwrap(), 7);
        assert_eq!(folder_id("overig-financieel").unwrap(), 8);
    }

    #[test]
    fn resolves_english_aliases() {
        assert_eq!(folder_id("purchase").unwrap(), 1);
        assert_eq!(folder_id("sales").unwrap(), 2);
        assert_eq!(folder_id("personnel").unwrap(), 4);
        assert_eq!(folder_id("tax").unwrap(), 5);
        assert_eq!(folder_id("other").unwrap(), 8);
    }

    #[test]
    fn resolves_case_insensitively_and_trims() {
        assert_eq!(folder_id("Verkoop").unwrap(), 2);
        assert_eq!(folder_id("  VERKOOP  ").unwrap(), 2);
    }

    #[test]
    fn passes_through_numeric_ids() {
        // The portal exposes folder IDs directly in its URLs (folder-contents/201),
        // including subfolders that have no name in this table.
        assert_eq!(folder_id("2").unwrap(), 2);
        assert_eq!(folder_id("201").unwrap(), 201);
        assert_eq!(folder_id("0").unwrap(), 0);
    }

    #[test]
    fn unknown_name_is_an_error_not_folder_zero() {
        // Regression: `unwrap_or(0)` made every unrecognised name silently return
        // the root folder, so `--folder bogusname` looked like a successful query.
        let err = folder_id("bogusname").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown folder"), "got: {msg}");
        assert!(
            msg.contains("verkoop"),
            "error should list valid names: {msg}"
        );
    }

    #[test]
    fn empty_name_is_an_error() {
        assert!(folder_id("").is_err());
    }
}

use std::path::PathBuf;

/// Represents the result of a Dynamic Range (DR) scan for a single audio file.
#[derive(Clone, Debug)]
pub struct DRResult {
    /// The calculated DR value as an integer. 
    /// `None` indicates either an error during scanning or that the file has not yet been scanned.
    pub dr_value: Option<u8>,
    /// The name of the audio file.
    pub filename: String,
    /// The full path to the audio file.
    pub path: PathBuf,
    /// A flag to indicate whether a scan has been attempted on this file.
    /// This helps distinguish between a pending file (`scanned: false`) and a file that was scanned but resulted in an error (`scanned: true`, `dr_value: None`).
    pub scanned: bool,
}

/// Holds the application's overall state, including settings and scan results.
pub struct AppState {
    /// If `true`, files removed from the list will also be deleted from the filesystem.
    pub delete_files: bool,
    /// If `true`, parent folders will be deleted if they become empty after a file is deleted.
    /// This is typically used in conjunction with `delete_files`.
    pub delete_folders: bool,
    /// A list of `DRResult` structs, representing all the files loaded into the application and their scan states.
    pub results: Vec<DRResult>,
}
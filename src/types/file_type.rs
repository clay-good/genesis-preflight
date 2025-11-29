use std::fmt;
use std::path::Path;

/// Enumeration of file types that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// Comma-separated values file
    Csv,
    /// Tab-separated values file
    Tsv,
    /// JSON data file
    Json,
    /// Plain text file
    Text,
    /// Markdown documentation file
    Markdown,
    /// Binary file (not text-based)
    Binary,
    /// Unknown or unsupported file type
    Unknown,
}

impl FileType {
    /// Infer file type from file extension
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "csv" => FileType::Csv,
            "tsv" => FileType::Tsv,
            "json" => FileType::Json,
            "txt" => FileType::Text,
            "md" | "markdown" => FileType::Markdown,
            "bin" | "dat" | "hdf5" | "h5" | "nc" | "netcdf" | "png" | "jpg" | "jpeg" | "pdf" => {
                FileType::Binary
            }
            _ => FileType::Unknown,
        }
    }

    /// Infer file type from path
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(Self::from_extension)
            .unwrap_or(FileType::Unknown)
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::Csv => write!(f, "CSV"),
            FileType::Tsv => write!(f, "TSV"),
            FileType::Json => write!(f, "JSON"),
            FileType::Text => write!(f, "Text"),
            FileType::Markdown => write!(f, "Markdown"),
            FileType::Binary => write!(f, "Binary"),
            FileType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl From<&str> for FileType {
    fn from(extension: &str) -> Self {
        Self::from_extension(extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_extension() {
        assert_eq!(FileType::from_extension("csv"), FileType::Csv);
        assert_eq!(FileType::from_extension("CSV"), FileType::Csv);
        assert_eq!(FileType::from_extension("tsv"), FileType::Tsv);
        assert_eq!(FileType::from_extension("json"), FileType::Json);
        assert_eq!(FileType::from_extension("md"), FileType::Markdown);
        assert_eq!(FileType::from_extension("txt"), FileType::Text);
        assert_eq!(FileType::from_extension("hdf5"), FileType::Binary);
        assert_eq!(FileType::from_extension("xyz"), FileType::Unknown);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", FileType::Csv), "CSV");
        assert_eq!(format!("{}", FileType::Json), "JSON");
        assert_eq!(format!("{}", FileType::Unknown), "Unknown");
    }
}

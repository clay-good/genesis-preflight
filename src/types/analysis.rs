use super::{ColumnType, FileType};

/// Information about a column in a CSV file
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column index (0-based)
    pub index: usize,
    /// Column name from header (if available)
    pub name: Option<String>,
    /// Inferred data type
    pub inferred_type: ColumnType,
    /// Number of null/empty values
    pub null_count: usize,
    /// Sample values from this column
    pub sample_values: Vec<String>,
}

impl ColumnInfo {
    /// Create a new ColumnInfo
    pub fn new(index: usize) -> Self {
        ColumnInfo {
            index,
            name: None,
            inferred_type: ColumnType::Unknown,
            null_count: 0,
            sample_values: Vec::new(),
        }
    }

    /// Set the column name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the inferred type
    pub fn with_type(mut self, inferred_type: ColumnType) -> Self {
        self.inferred_type = inferred_type;
        self
    }

    /// Set the null count
    pub fn with_null_count(mut self, null_count: usize) -> Self {
        self.null_count = null_count;
        self
    }

    /// Add a sample value
    pub fn add_sample(mut self, value: String) -> Self {
        if self.sample_values.len() < 5 && !self.sample_values.contains(&value) {
            self.sample_values.push(value);
        }
        self
    }
}

/// Analysis result for a CSV file
#[derive(Debug, Clone)]
pub struct CsvAnalysis {
    /// Detected delimiter character
    pub delimiter: char,
    /// Whether the file has a header row
    pub has_header: bool,
    /// Number of columns
    pub column_count: usize,
    /// Number of data rows (excluding header)
    pub row_count: usize,
    /// Information about each column
    pub columns: Vec<ColumnInfo>,
}

impl CsvAnalysis {
    /// Create a new CsvAnalysis
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvAnalysis {
            delimiter,
            has_header,
            column_count: 0,
            row_count: 0,
            columns: Vec::new(),
        }
    }
}

/// Type of JSON root element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonRootType {
    /// JSON object at root
    Object,
    /// JSON array at root
    Array,
}

/// Analysis result for a JSON file
#[derive(Debug, Clone)]
pub struct JsonAnalysis {
    /// Whether the JSON is syntactically valid
    pub is_valid: bool,
    /// Type of the root element
    pub root_type: JsonRootType,
    /// Top-level keys (if root is an object)
    pub top_level_keys: Vec<String>,
}

impl JsonAnalysis {
    /// Create a new JsonAnalysis
    pub fn new(is_valid: bool, root_type: JsonRootType) -> Self {
        JsonAnalysis {
            is_valid,
            root_type,
            top_level_keys: Vec::new(),
        }
    }

    /// Create an analysis for an invalid JSON file
    pub fn invalid() -> Self {
        JsonAnalysis {
            is_valid: false,
            root_type: JsonRootType::Object,
            top_level_keys: Vec::new(),
        }
    }
}

/// Analysis result for a text file
#[derive(Debug, Clone)]
pub struct TextAnalysis {
    /// Number of lines in the file
    pub line_count: usize,
    /// Number of words in the file
    pub word_count: usize,
    /// Whether this appears to be documentation
    pub is_documentation: bool,
    /// List of encoding issues found
    pub encoding_issues: Vec<String>,
}

impl TextAnalysis {
    /// Create a new TextAnalysis
    pub fn new() -> Self {
        TextAnalysis {
            line_count: 0,
            word_count: 0,
            is_documentation: false,
            encoding_issues: Vec::new(),
        }
    }
}

impl Default for TextAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of binary file detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryType {
    /// HDF5 scientific data format
    Hdf5,
    /// NetCDF scientific data format
    Netcdf,
    /// PNG image
    Png,
    /// JPEG image
    Jpeg,
    /// PDF document
    Pdf,
    /// Unknown binary type
    Unknown,
}

/// Analysis result for a binary file
#[derive(Debug, Clone)]
pub struct BinaryAnalysis {
    /// Detected binary file type
    pub binary_type: BinaryType,
}

impl BinaryAnalysis {
    /// Create a new BinaryAnalysis
    pub fn new(binary_type: BinaryType) -> Self {
        BinaryAnalysis { binary_type }
    }
}

/// Result of analyzing a file
#[derive(Debug, Clone)]
pub enum AnalysisResult {
    /// CSV analysis result
    Csv(CsvAnalysis),
    /// JSON analysis result
    Json(JsonAnalysis),
    /// Text analysis result
    Text(TextAnalysis),
    /// Binary analysis result
    Binary(BinaryAnalysis),
    /// File was not analyzed
    NotAnalyzed,
}

/// Summary of an entire dataset
#[derive(Debug, Clone)]
pub struct DatasetSummary {
    /// Total number of files
    pub total_files: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Count of each file type
    pub file_type_counts: Vec<(FileType, usize)>,
    /// Timestamp when scan was performed
    pub scan_timestamp: String,
}

impl DatasetSummary {
    /// Create a new DatasetSummary
    pub fn new() -> Self {
        DatasetSummary {
            total_files: 0,
            total_size: 0,
            file_type_counts: Vec::new(),
            scan_timestamp: String::new(),
        }
    }

    /// Format total size as human-readable string
    pub fn format_size(&self) -> String {
        let size = self.total_size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl Default for DatasetSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_info_builder() {
        let col = ColumnInfo::new(0)
            .with_name("temperature".to_string())
            .with_type(ColumnType::Float)
            .with_null_count(5)
            .add_sample("23.5".to_string());

        assert_eq!(col.index, 0);
        assert_eq!(col.name, Some("temperature".to_string()));
        assert_eq!(col.inferred_type, ColumnType::Float);
        assert_eq!(col.null_count, 5);
        assert_eq!(col.sample_values.len(), 1);
    }

    #[test]
    fn test_csv_analysis_creation() {
        let analysis = CsvAnalysis::new(',', true);
        assert_eq!(analysis.delimiter, ',');
        assert!(analysis.has_header);
        assert_eq!(analysis.column_count, 0);
    }

    #[test]
    fn test_json_analysis_invalid() {
        let analysis = JsonAnalysis::invalid();
        assert!(!analysis.is_valid);
    }

    #[test]
    fn test_dataset_summary_size_formatting() {
        let mut summary = DatasetSummary::new();

        summary.total_size = 500;
        assert_eq!(summary.format_size(), "500 B");

        summary.total_size = 2048;
        assert_eq!(summary.format_size(), "2.00 KB");

        summary.total_size = 2 * 1024 * 1024;
        assert_eq!(summary.format_size(), "2.00 MB");

        summary.total_size = 3 * 1024 * 1024 * 1024;
        assert_eq!(summary.format_size(), "3.00 GB");
    }
}

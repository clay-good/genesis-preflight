use std::path::PathBuf;

/// Command to execute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Scan and validate dataset only
    Scan,
    /// Scan, validate, and generate missing documentation
    Generate,
    /// Generate machine-readable JSON report
    Report,
}

/// Runtime configuration for the tool
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the dataset to scan
    pub target_path: PathBuf,
    /// Optional output directory for generated files
    pub output_dir: Option<PathBuf>,
    /// Command to execute
    pub command: Command,
    /// Enable verbose output
    pub verbose: bool,
    /// Enable quiet mode (minimal output)
    pub quiet: bool,
    /// Skip SHA-256 hash calculation
    pub skip_hash: bool,
    /// Generate JSON output (for report command)
    pub json_output: bool,
}

impl Config {
    /// Create a new configuration
    pub fn new(target_path: PathBuf, command: Command) -> Self {
        Config {
            target_path,
            output_dir: None,
            command,
            verbose: false,
            quiet: false,
            skip_hash: false,
            json_output: false,
        }
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, output_dir: PathBuf) -> Self {
        self.output_dir = Some(output_dir);
        self
    }

    /// Enable verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable quiet mode
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Skip hash calculation
    pub fn with_skip_hash(mut self, skip_hash: bool) -> Self {
        self.skip_hash = skip_hash;
        self
    }

    /// Enable JSON output
    pub fn with_json_output(mut self, json_output: bool) -> Self {
        self.json_output = json_output;
        self
    }

    /// Get the effective output directory (defaults to target_path if not set)
    pub fn get_output_dir(&self) -> &PathBuf {
        self.output_dir.as_ref().unwrap_or(&self.target_path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            target_path: PathBuf::from("."),
            output_dir: None,
            command: Command::Scan,
            verbose: false,
            quiet: false,
            skip_hash: false,
            json_output: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.command, Command::Scan);
        assert!(!config.verbose);
        assert!(!config.quiet);
        assert!(!config.skip_hash);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new(PathBuf::from("/test"), Command::Generate)
            .with_verbose(true)
            .with_skip_hash(true);

        assert_eq!(config.command, Command::Generate);
        assert!(config.verbose);
        assert!(config.skip_hash);
    }

    #[test]
    fn test_output_dir_default() {
        let config = Config::new(PathBuf::from("/test"), Command::Scan);
        assert_eq!(config.get_output_dir(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_output_dir_custom() {
        let config = Config::new(PathBuf::from("/test"), Command::Scan)
            .with_output_dir(PathBuf::from("/output"));
        assert_eq!(config.get_output_dir(), &PathBuf::from("/output"));
    }
}

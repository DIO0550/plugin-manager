/// Error codes with category prefix
///
/// Categories:
/// - NET: Network connectivity errors
/// - API: GitHub/Repository API errors
/// - IO: File system operations
/// - CFG: Configuration parsing/validation
/// - PLG: Plugin-related errors
/// - MKT: Marketplace operations
/// - TUI: Terminal UI errors
/// - VAL: Input validation errors
/// - INT: Unexpected internal errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Network errors (NET001-NET099)
    /// Connection failed
    Net001,
    /// Request timeout
    Net002,

    // API errors (API001-API099)
    /// Rate limit exceeded
    Api001,
    /// Authentication failed
    Api002,
    /// Resource not found
    Api003,
    /// Server error (5xx)
    Api004,

    // I/O errors (IO001-IO099)
    /// File not found
    Io001,
    /// Permission denied
    Io002,
    /// Disk full
    Io003,

    // Config errors (CFG001-CFG099)
    /// Invalid config format
    Cfg001,
    /// Missing required field
    Cfg002,

    // Plugin errors (PLG001-PLG099)
    /// Plugin not found
    Plg001,
    /// Invalid manifest
    Plg002,
    /// Ambiguous plugin name
    Plg003,

    // Marketplace errors (MKT001-MKT099)
    /// Marketplace not found
    Mkt001,

    // TUI errors (TUI001-TUI099)
    /// Terminal initialization failed
    Tui001,

    // Validation errors (VAL001-VAL099)
    /// Invalid argument
    Val001,
    /// Invalid repository format
    Val002,

    // Internal errors (INT001-INT099)
    /// Unexpected internal error
    Int001,
}

impl ErrorCode {
    /// Returns the error code string (e.g., "NET001")
    pub fn as_str(&self) -> &'static str {
        match self {
            // Network
            ErrorCode::Net001 => "NET001",
            ErrorCode::Net002 => "NET002",
            // API
            ErrorCode::Api001 => "API001",
            ErrorCode::Api002 => "API002",
            ErrorCode::Api003 => "API003",
            ErrorCode::Api004 => "API004",
            // I/O
            ErrorCode::Io001 => "IO001",
            ErrorCode::Io002 => "IO002",
            ErrorCode::Io003 => "IO003",
            // Config
            ErrorCode::Cfg001 => "CFG001",
            ErrorCode::Cfg002 => "CFG002",
            // Plugin
            ErrorCode::Plg001 => "PLG001",
            ErrorCode::Plg002 => "PLG002",
            ErrorCode::Plg003 => "PLG003",
            // Marketplace
            ErrorCode::Mkt001 => "MKT001",
            // TUI
            ErrorCode::Tui001 => "TUI001",
            // Validation
            ErrorCode::Val001 => "VAL001",
            ErrorCode::Val002 => "VAL002",
            // Internal
            ErrorCode::Int001 => "INT001",
        }
    }

    /// Returns the general cause description
    pub fn cause(&self) -> &'static str {
        match self {
            // Network
            ErrorCode::Net001 => "Unable to establish network connection to the server",
            ErrorCode::Net002 => "The request timed out while waiting for a response",
            // API
            ErrorCode::Api001 => "API rate limit has been exceeded",
            ErrorCode::Api002 => "Authentication failed or access denied",
            ErrorCode::Api003 => "The requested resource was not found",
            ErrorCode::Api004 => "The server encountered an internal error",
            // I/O
            ErrorCode::Io001 => "The specified file or directory was not found",
            ErrorCode::Io002 => "Permission denied when accessing the file or directory",
            ErrorCode::Io003 => "Insufficient disk space to complete the operation",
            // Config
            ErrorCode::Cfg001 => "The configuration file has an invalid format",
            ErrorCode::Cfg002 => "A required configuration field is missing",
            // Plugin
            ErrorCode::Plg001 => "The specified plugin was not found",
            ErrorCode::Plg002 => "The plugin manifest is invalid or corrupted",
            ErrorCode::Plg003 => "Multiple plugins with the same name were found",
            // Marketplace
            ErrorCode::Mkt001 => "The specified marketplace was not found",
            // TUI
            ErrorCode::Tui001 => "Failed to initialize the terminal interface",
            // Validation
            ErrorCode::Val001 => "An invalid argument was provided",
            ErrorCode::Val002 => "Invalid repository format",
            // Internal
            ErrorCode::Int001 => "An unexpected internal error occurred",
        }
    }

    /// Returns remediation steps
    pub fn remediation(&self) -> &'static str {
        match self {
            // Network
            ErrorCode::Net001 => "1. Check your internet connection\n2. Verify the URL is correct\n3. Try again later if the server is down",
            ErrorCode::Net002 => "1. Check your internet connection speed\n2. Try again with a longer timeout\n3. The server may be overloaded, try later",
            // API
            ErrorCode::Api001 => "1. Wait a few minutes before retrying\n2. Check your API usage quota\n3. Consider using authenticated requests",
            ErrorCode::Api002 => "1. Verify your credentials are correct\n2. Check if your token has expired\n3. Ensure you have the required permissions",
            ErrorCode::Api003 => "1. Verify the resource name is correct\n2. Check if the resource still exists\n3. Ensure you have access to the resource",
            ErrorCode::Api004 => "1. Wait a few minutes and retry\n2. Check the service status page\n3. Report the issue if it persists",
            // I/O
            ErrorCode::Io001 => "1. Verify the file path is correct\n2. Check if the file was moved or deleted\n3. Ensure the path exists",
            ErrorCode::Io002 => "1. Check file/directory permissions\n2. Run with appropriate privileges\n3. Verify ownership of the resource",
            ErrorCode::Io003 => "1. Free up disk space\n2. Move files to another drive\n3. Clean up temporary files",
            // Config
            ErrorCode::Cfg001 => "1. Check the configuration file syntax\n2. Validate against the expected format\n3. Restore from a backup if corrupted",
            ErrorCode::Cfg002 => "1. Add the missing required field\n2. Check the documentation for required fields\n3. Use default configuration as reference",
            // Plugin
            ErrorCode::Plg001 => "1. Verify the plugin name is correct\n2. Check if the plugin is installed\n3. Use 'plm list' to see available plugins",
            ErrorCode::Plg002 => "1. Re-download the plugin\n2. Check if the plugin is compatible\n3. Report the issue to the plugin author",
            ErrorCode::Plg003 => "1. Use the full plugin identifier\n2. Specify the marketplace explicitly\n3. Use 'plm info' to see available options",
            // Marketplace
            ErrorCode::Mkt001 => "1. Verify the marketplace name\n2. Register the marketplace first\n3. Use 'plm marketplace list' to see available marketplaces",
            // TUI
            ErrorCode::Tui001 => "1. Ensure your terminal supports the required features\n2. Try a different terminal emulator\n3. Check terminal configuration",
            // Validation
            ErrorCode::Val001 => "1. Check the argument format\n2. Refer to the command help\n3. Use 'plm --help' for usage information",
            ErrorCode::Val002 => "1. Use the format 'owner/repo' or 'owner/repo@ref'\n2. Check for typos in the repository name\n3. Verify the repository exists",
            // Internal
            ErrorCode::Int001 => "1. Try the operation again\n2. Check for updates to plm\n3. Report the issue with debug logs",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_error_codes_have_valid_string() {
        let codes = [
            ErrorCode::Net001,
            ErrorCode::Net002,
            ErrorCode::Api001,
            ErrorCode::Api002,
            ErrorCode::Api003,
            ErrorCode::Api004,
            ErrorCode::Io001,
            ErrorCode::Io002,
            ErrorCode::Io003,
            ErrorCode::Cfg001,
            ErrorCode::Cfg002,
            ErrorCode::Plg001,
            ErrorCode::Plg002,
            ErrorCode::Plg003,
            ErrorCode::Mkt001,
            ErrorCode::Tui001,
            ErrorCode::Val001,
            ErrorCode::Val002,
            ErrorCode::Int001,
        ];

        for code in codes {
            let s = code.as_str();
            assert!(!s.is_empty(), "Error code string should not be empty");
            assert!(
                s.len() >= 5 && s.len() <= 6,
                "Error code string should be 5-6 characters: {}",
                s
            );
        }
    }

    #[test]
    fn all_error_codes_have_cause() {
        let codes = [
            ErrorCode::Net001,
            ErrorCode::Net002,
            ErrorCode::Api001,
            ErrorCode::Api002,
            ErrorCode::Api003,
            ErrorCode::Api004,
            ErrorCode::Io001,
            ErrorCode::Io002,
            ErrorCode::Io003,
            ErrorCode::Cfg001,
            ErrorCode::Cfg002,
            ErrorCode::Plg001,
            ErrorCode::Plg002,
            ErrorCode::Plg003,
            ErrorCode::Mkt001,
            ErrorCode::Tui001,
            ErrorCode::Val001,
            ErrorCode::Val002,
            ErrorCode::Int001,
        ];

        for code in codes {
            let cause = code.cause();
            assert!(
                !cause.is_empty(),
                "Cause should not be empty for {:?}",
                code
            );
        }
    }

    #[test]
    fn all_error_codes_have_remediation() {
        let codes = [
            ErrorCode::Net001,
            ErrorCode::Net002,
            ErrorCode::Api001,
            ErrorCode::Api002,
            ErrorCode::Api003,
            ErrorCode::Api004,
            ErrorCode::Io001,
            ErrorCode::Io002,
            ErrorCode::Io003,
            ErrorCode::Cfg001,
            ErrorCode::Cfg002,
            ErrorCode::Plg001,
            ErrorCode::Plg002,
            ErrorCode::Plg003,
            ErrorCode::Mkt001,
            ErrorCode::Tui001,
            ErrorCode::Val001,
            ErrorCode::Val002,
            ErrorCode::Int001,
        ];

        for code in codes {
            let remediation = code.remediation();
            assert!(
                !remediation.is_empty(),
                "Remediation should not be empty for {:?}",
                code
            );
        }
    }

    #[test]
    fn error_code_format_matches_pattern() {
        let codes = [
            (ErrorCode::Net001, "NET"),
            (ErrorCode::Net002, "NET"),
            (ErrorCode::Api001, "API"),
            (ErrorCode::Api002, "API"),
            (ErrorCode::Api003, "API"),
            (ErrorCode::Api004, "API"),
            (ErrorCode::Io001, "IO0"),
            (ErrorCode::Io002, "IO0"),
            (ErrorCode::Io003, "IO0"),
            (ErrorCode::Cfg001, "CFG"),
            (ErrorCode::Cfg002, "CFG"),
            (ErrorCode::Plg001, "PLG"),
            (ErrorCode::Plg002, "PLG"),
            (ErrorCode::Plg003, "PLG"),
            (ErrorCode::Mkt001, "MKT"),
            (ErrorCode::Tui001, "TUI"),
            (ErrorCode::Val001, "VAL"),
            (ErrorCode::Val002, "VAL"),
            (ErrorCode::Int001, "INT"),
        ];

        for (code, expected_prefix) in codes {
            let s = code.as_str();
            assert!(
                s.starts_with(expected_prefix),
                "Error code {} should start with {}",
                s,
                expected_prefix
            );
        }
    }
}

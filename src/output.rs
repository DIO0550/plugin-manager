use owo_colors::OwoColorize;

pub struct CommandSummary {
    pub prefix: String,
    pub message: String,
}

impl CommandSummary {
    pub fn format(success: usize, failure: usize) -> Self {
        match (success, failure) {
            (_, f) if f > 0 => Self {
                prefix: "✗".red().to_string(),
                message: format!("{} succeeded, {} failed", success.green(), f.red()),
            },
            (s, _) if s > 0 => Self {
                prefix: "✓".green().to_string(),
                message: format!("{} component(s) placed", s.green()),
            },
            _ => Self {
                prefix: "•".yellow().to_string(),
                message: "No matching components found".to_string(),
            },
        }
    }
}

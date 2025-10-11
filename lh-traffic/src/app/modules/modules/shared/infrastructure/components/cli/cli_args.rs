use std::sync::Arc;
use once_cell::sync::Lazy;

/// CLI arguments parser
/// Equivalent to TypeScript's LzCliArgs
pub struct CliArgs {
    args: Vec<String>,
}

static INSTANCE: Lazy<Arc<CliArgs>> = Lazy::new(|| {
    let args: Vec<String> = std::env::args().skip(1).collect();
    Arc::new(CliArgs { args })
});

impl CliArgs {
    pub fn instance() -> Arc<CliArgs> {
        INSTANCE.clone()
    }

    /// Get argument at index
    pub fn get_arg(&self, index: usize) -> Option<&String> {
        self.args.get(index)
    }

    /// Get all arguments
    pub fn get_all_args(&self) -> &[String] {
        &self.args
    }

    /// Check if argument exists
    pub fn has_arg(&self, index: usize) -> bool {
        self.args.get(index).is_some()
    }
}

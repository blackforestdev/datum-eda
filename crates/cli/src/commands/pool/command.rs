//! Library search and attribution for the native symbols distributed with Datum.
use crate::*;

impl PoolCommands {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        match self {
            Self::Search { query, libraries } => {
                let results = search_pool(&query, &libraries)?;
                Ok((render_output(format, &results), 0))
            }
            Self::License => {
                let notice = eda_engine::import::library_bundle::COPYRIGHT;
                let output = match format {
                    OutputFormat::Text => notice.to_owned(),
                    OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                        "library": "KiCad symbol library", "notice": notice
                    }))?,
                };
                Ok((output, 0))
            }
        }
    }
}

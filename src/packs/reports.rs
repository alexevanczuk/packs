use std::io;

use serde_json::json;

use super::Configuration;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub(crate) enum ReportFormat {
    Csv,
    Json,
}

pub(crate) fn recorded_violations(
    configuration: &Configuration,
    report_format: ReportFormat,
    writer: &mut impl io::Write,
) -> anyhow::Result<()> {
    let recorded_violations = &configuration.pack_set.all_violations;

    match report_format {
        ReportFormat::Csv => {
            let mut csv_writer = csv::Writer::from_writer(writer);
            for violation in recorded_violations {
                csv_writer.serialize(violation)?;
            }
            csv_writer.flush()?;
        }
        ReportFormat::Json => {
            serde_json::to_writer(
                writer,
                &json!({
                    "violations": recorded_violations,
                }),
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::packs::configuration;

    use super::*;

    #[test]
    fn test_recorded_violations_json() -> anyhow::Result<()> {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/contains_stale_violations")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )?;
        let mut writer = Vec::new();
        recorded_violations(&configuration, ReportFormat::Json, &mut writer)?;
        let parsed_json: serde_json::Value = serde_json::from_slice(&writer)?;
        assert_eq!(parsed_json["violations"].as_array().unwrap().len(), 4);
        Ok(())
    }

    #[test]
    fn test_recorded_violations_csv() -> anyhow::Result<()> {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/contains_stale_violations")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )?;
        let mut writer = Vec::new();
        recorded_violations(&configuration, ReportFormat::Csv, &mut writer)?;
        let csv_string = String::from_utf8(writer)?;
        assert_eq!(csv_string.lines().count(), 5);
        assert_eq!(csv_string.lines().nth(0).unwrap(), "violation_type,strict,file,constant_name,referencing_pack_name,defining_pack_name");
        Ok(())
    }
}

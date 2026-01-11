use chrono::NaiveDate;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::Result;
use crate::journal::template;

pub fn ensure_year_dir(year: u32, base_path: &Path) -> Result<PathBuf> {
    let year_path = base_path.join(year.to_string());
    fs::create_dir_all(&year_path)?;
    Ok(year_path)
}

pub fn ensure_month_dir(year: u32, month: u32, base_path: &Path) -> Result<PathBuf> {
    let month_path = base_path
        .join(year.to_string())
        .join(format!("{:02}", month));
    fs::create_dir_all(&month_path)?;
    Ok(month_path)
}

pub fn create_month_readme(year: u32, month: u32, base_path: &Path, config: &Config) -> Result<()> {
    let month_path = base_path
        .join(year.to_string())
        .join(format!("{:02}", month));
    let readme_path = month_path.join("README.md");

    // Don't overwrite existing README
    if readme_path.exists() {
        return Ok(());
    }

    // Load and apply month template
    let template_content = template::load_month_template(&config.month_template_path)?;
    let content = template::apply_month_variables(&template_content, year, month);

    fs::write(readme_path, content)?;
    Ok(())
}

pub fn create_year_readme(year: u32, base_path: &Path, config: &Config) -> Result<()> {
    let year_path = base_path.join(year.to_string());
    let readme_path = year_path.join("README.md");

    // Don't overwrite existing README
    if readme_path.exists() {
        return Ok(());
    }

    // Load and apply year template
    let template_content = template::load_year_template(&config.year_template_path)?;
    let content = template::apply_year_variables(&template_content, year);

    fs::write(readme_path, content)?;
    Ok(())
}

pub fn get_entry_path(date: NaiveDate, base_path: &Path) -> PathBuf {
    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let day = date.format("%d").to_string();

    base_path.join(year).join(month).join(format!("{}.md", day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_entry_path() {
        let date = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();
        let path = get_entry_path(date, Path::new("journal"));
        assert_eq!(path, PathBuf::from("journal/2025/12/29.md"));
    }
}

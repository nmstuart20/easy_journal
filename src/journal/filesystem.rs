use chrono::NaiveDate;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

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

pub fn create_month_readme(year: u32, month: u32, base_path: &Path) -> Result<()> {
    let month_path = base_path
        .join(year.to_string())
        .join(format!("{:02}", month));
    let readme_path = month_path.join("README.md");

    // Don't overwrite existing README
    if readme_path.exists() {
        return Ok(());
    }

    let month_name = get_month_name(month);
    let content = format!(
        "# {} {}\n\n## Goals for this month \n - [ ] \n\n---\n\n",
        month_name, year
    );

    fs::write(readme_path, content)?;
    Ok(())
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
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
    fn test_get_month_name() {
        assert_eq!(get_month_name(1), "January");
        assert_eq!(get_month_name(12), "December");
    }

    #[test]
    fn test_get_entry_path() {
        let date = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();
        let path = get_entry_path(date, Path::new("journal"));
        assert_eq!(path, PathBuf::from("journal/2025/12/29.md"));
    }
}

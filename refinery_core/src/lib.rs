use regex::Regex;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_migration_files(
    location: impl AsRef<Path>,
) -> Result<impl Iterator<Item = PathBuf>, String> {
    let re = Regex::new(r"^(V)(\d+(?:\.\d+)?)__(\w+)\.rs$").unwrap();
    let location: &Path = location.as_ref();
    let location = location.canonicalize().map_err(|_| String::new())?;

    let file_paths = WalkDir::new(location)
        .into_iter()
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(
            move |entry| match entry.file_name().and_then(OsStr::to_str) {
                Some(file_name) => re.is_match(file_name),
                None => false,
            },
        );

    Ok(file_paths)
}

#[derive(Clone, Debug)]
pub enum MigrationPrefix {
    Versioned,
}

#[derive(Clone, Debug)]
pub struct Migration {
    pub name: String,
    pub version: usize,
    pub prefix: MigrationPrefix,
    pub sql: String,
}

impl Migration {
    pub fn from_filename(name: &str, sql: &str) -> Result<Migration, String> {
        let captures = Regex::new(r"^(V)(\d+(?:\.\d+)?)__(\w+)")
            .unwrap()
            .captures(name)
            .filter(|caps| caps.len() == 4)
            .ok_or_else(String::new)?;
        let version = captures[2].parse().map_err(|_| String::new())?;

        let name = (&captures[3]).into();
        let prefix = match &captures[1] {
            "V" => MigrationPrefix::Versioned,
            _ => unreachable!(),
        };

        Ok(Migration {
            name,
            version,
            sql: sql.into(),
            prefix,
        })
    }
}

use chrono::{DateTime, Local};
use regex::Regex;
use siphasher::sip::SipHasher13;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::Error;

// regex used to match file names
pub fn file_match_re() -> Regex {
    Regex::new(r"^(V)(\d+(?:\.\d+)?)__(\w+)").unwrap()
}

lazy_static::lazy_static! {
    static ref RE: regex::Regex = file_match_re();
}

/// An enum set that represents the prefix for the Migration, at the moment only Versioned is supported
#[derive(Clone, Debug)]
pub enum MigrationPrefix {
    Versioned,
}

/// An enum set that represents the target version up to which refinery should migrate, it is used by [Runner]
#[derive(Clone, Copy)]
pub enum Target {
    Latest,
    Version(usize),
}

/// Represents a schema migration to be run on the database,
/// this struct is used by the [`embed_migrations!`] and [`include_migration_mods!`] macros to gather migration files
/// and shouldn't be needed by the user
///
/// [`embed_migrations!`]: macro.embed_migrations.html
/// [`include_migration_mods!`]: macro.include_migration_mods.html
#[derive(Clone, Debug)]
pub struct Migration {
    pub name: String,
    pub version: usize,
    pub prefix: MigrationPrefix,
    pub sql: String,
}

impl Migration {
    pub fn from_filename(name: &str, sql: &str) -> Result<Migration, Error> {
        let captures = RE
            .captures(name)
            .filter(|caps| caps.len() == 4)
            .ok_or(Error::InvalidName)?;
        let version = captures[2].parse().map_err(|_| Error::InvalidVersion)?;

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

    pub fn checksum(&self) -> u64 {
        // Previously, `std::collections::hash_map::DefaultHasher` was used
        // to calculate the checksum and the implementation at that time
        // was SipHasher13. However, that implementation is not guaranteed:
        // > The internal algorithm is not specified, and so it and its
        // > hashes should not be relied upon over releases.
        // We now explicitly use SipHasher13 to both remain compatible with
        // existing migrations and prevent breaking from possible future
        // changes to `DefaultHasher`.
        let mut hasher = SipHasher13::new();
        self.name.hash(&mut hasher);
        self.version.hash(&mut hasher);
        self.sql.hash(&mut hasher);
        hasher.finish()
    }
}

impl fmt::Display for Migration {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "V{}__{}", self.version, self.name)
    }
}

impl Eq for Migration {}

impl PartialEq for Migration {
    fn eq(&self, other: &Migration) -> bool {
        self.version == other.version
            && self.name == other.name
            && self.checksum() == other.checksum()
    }
}

impl Ord for Migration {
    fn cmp(&self, other: &Migration) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for Migration {
    fn partial_cmp(&self, other: &Migration) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct AppliedMigration {
    pub name: String,
    pub version: usize,
    pub applied_on: DateTime<Local>,
    pub checksum: String,
}

impl Eq for AppliedMigration {}

impl PartialEq for AppliedMigration {
    fn eq(&self, other: &AppliedMigration) -> bool {
        self.version == other.version && self.name == other.name && self.checksum == other.checksum
    }
}

impl fmt::Display for AppliedMigration {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "V{}__{}", self.version, self.name)
    }
}

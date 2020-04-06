use regex::Regex;

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

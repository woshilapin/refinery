use regex::Regex;

#[derive(Clone, Debug)]
pub struct Migration {
    pub name: String,
    pub sql: String,
}

impl Migration {
    pub fn from_filename(name: &str, sql: &str) -> Result<Migration, String> {
        let captures = Regex::new(r"^([^m]\w+)")
            .unwrap()
            .captures(name)
            .filter(|caps| caps.len() == 2)
            .ok_or_else(String::new)?;

        let name = (&captures[1]).into();

        Ok(Migration {
            name,
            sql: sql.into(),
        })
    }
}

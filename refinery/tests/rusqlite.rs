use barrel::backend::Sqlite as Sql;
mod mod_migrations;

#[cfg(feature = "rusqlite")]
mod rusqlite {
    use super::mod_migrations;
    use assert_cmd::prelude::*;
    use chrono::{DateTime, Local};
    use predicates::str::contains;
    use refinery::{
        config::{migrate_from_config, Config, ConfigDbType},
        Error, Migrate, Migration, Target,
    };
    use refinery_core::rusqlite::{Connection, OptionalExtension, NO_PARAMS};
    use std::fs::{self, File};
    use std::process::Command;

    fn run_test<T>(test: T)
    where
        T: FnOnce() + std::panic::UnwindSafe,
    {
        let filepath = "tests/db.sql";
        File::create(filepath).unwrap();

        let result = std::panic::catch_unwind(|| test());

        fs::remove_file(filepath).unwrap();

        assert!(result.is_ok())
    }

    fn get_migrations() -> Vec<Migration> {
        vec![]
    }

    #[test]
    fn mod_applies_migration() {
        let mut conn = Connection::open_in_memory().unwrap();

        mod_migrations::migrations::runner().run(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO persons (name, city) VALUES (?, ?)",
            &[&"John Legend", &"New York"],
        )
        .unwrap();
        let (name, city): (String, String) = conn
            .query_row("SELECT name, city FROM persons", NO_PARAMS, |row| {
                Ok((row.get(0).unwrap(), row.get(1).unwrap()))
            })
            .unwrap();
        assert_eq!("John Legend", name);
        assert_eq!("New York", city);
    }

    #[test]
    fn mod_updates_schema_history() {
        let mut conn = Connection::open_in_memory().unwrap();

        mod_migrations::migrations::runner().run(&mut conn).unwrap();

        let current: u32 = conn
            .query_row(
                "SELECT MAX(version) FROM refinery_schema_history",
                NO_PARAMS,
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(4, current);

        let applied_on: DateTime<Local> = conn
            .query_row(
                "SELECT applied_on FROM refinery_schema_history where version=(SELECT MAX(version) from refinery_schema_history)",
                NO_PARAMS,
                |row| {
                    let applied_on: String = row.get(0).unwrap();
                    Ok(DateTime::parse_from_rfc3339(&applied_on).unwrap().with_timezone(&Local))
                }
            )
            .unwrap();
        assert_eq!(Local::today(), applied_on.date());
    }

    #[test]
    fn aborts_on_missing_migration_on_filesystem() {
        let mut conn = Connection::open_in_memory().unwrap();

        mod_migrations::migrations::runner().run(&mut conn).unwrap();

        let migration = Migration::from_filename(
            "V4__add_year_field_to_cars",
            &"ALTER TABLE cars ADD year INTEGER;",
        )
        .unwrap();
        let err = conn
            .migrate(&[migration], true, true, false, Target::Latest)
            .unwrap_err();

        match err {
            Error::MissingVersion(missing) => {
                assert_eq!(1, missing.version);
                assert_eq!("initial", missing.name);
            }
            _ => panic!("failed test"),
        }
    }

    #[test]
    fn aborts_on_divergent_migration() {
        let mut conn = Connection::open_in_memory().unwrap();

        mod_migrations::migrations::runner().run(&mut conn).unwrap();

        let migration = Migration::from_filename(
            "V2__add_year_field_to_cars",
            &"ALTER TABLE cars ADD year INTEGER;",
        )
        .unwrap();
        let err = conn
            .migrate(&[migration.clone()], true, false, false, Target::Latest)
            .unwrap_err();

        match err {
            Error::DivergentVersion(applied, divergent) => {
                assert_eq!(migration, divergent);
                assert_eq!(2, applied.version);
                assert_eq!("add_cars_table", applied.name);
            }
            _ => panic!("failed test"),
        }
    }

    #[test]
    fn migrates_from_config() {
        let db = tempfile::NamedTempFile::new_in(".").unwrap();
        let config = Config::new(ConfigDbType::Sqlite).set_db_path(db.path().to_str().unwrap());

        let migrations = get_migrations();
        migrate_from_config(&config, false, true, true, &migrations).unwrap();
    }

    #[test]
    fn migrates_from_cli() {
        run_test(|| {
            Command::new("refinery")
                .args(&[
                    "migrate",
                    "-c",
                    "tests/sqlite_refinery.toml",
                    "files",
                    "-p",
                    "tests/sql_migrations",
                ])
                .unwrap()
                .assert()
                .stdout(contains("applying migration: V4__add_year_to_motos_table"));
        })
    }
}

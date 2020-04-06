#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct Migration {
    pub name: String,
}
pub use refinery_macros::include_migration_mods;

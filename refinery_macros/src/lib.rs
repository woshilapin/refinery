//! Contains Refinery macros that are used to import and embed migration files.
#![recursion_limit = "128"]
//TODO remove when previous version is 1.42
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use quote::ToTokens;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use syn::{parse_macro_input, Ident, LitStr};

use refinery_core::{find_migration_files, MigrationType};

pub(crate) fn crate_root() -> PathBuf {
    let crate_root = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not present");
    PathBuf::from(crate_root)
}

fn migration_fn_quoted<T: ToTokens>(_migrations: Vec<T>) -> TokenStream2 {
    let result = quote! {
        use refinery::{Migration, Runner};
        pub fn runner() -> Runner {
            let quoted_migrations: Vec<(&str, String)> = vec![#(#_migrations),*];
            let mut migrations: Vec<Migration> = Vec::new();
            for module in quoted_migrations.into_iter() {
                migrations.push(Migration::from_filename(module.0, &module.1).unwrap());
            }
            Runner::new(&migrations)
        }
    };
    result
}

#[proc_macro]
pub fn include_migration_mods(input: TokenStream) -> TokenStream {
    let location = if input.is_empty() {
        crate_root().join("src").join("migrations")
    } else {
        let location: LitStr = parse_macro_input!(input);
        crate_root().join(location.value())
    };

    let migration_mod_names = find_migration_files(location, MigrationType::Mod)
        .expect("error getting migration files")
        .filter_map(|entry| entry.file_stem().and_then(OsStr::to_str).map(String::from));

    let mut migrations_mods = Vec::new();
    let mut _migrations = Vec::new();

    for migration in migration_mod_names {
        let ident = Ident::new(migration.as_str(), Span2::call_site());
        let mig_mod = quote! {pub mod #ident;};
        _migrations.push(quote! {(#migration, #ident::migration())});
        migrations_mods.push(mig_mod);
    }

    let fnq = migration_fn_quoted(_migrations);
    let result = quote! {
        #(#migrations_mods)*

        #fnq
    };
    result.into()
}

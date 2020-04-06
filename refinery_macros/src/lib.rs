//! Contains Refinery macros that are used to import and embed migration files.
#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use std::ffi::OsStr;
use std::path::PathBuf;
use syn::Ident;

#[proc_macro]
pub fn include_migration_mods(_: TokenStream) -> TokenStream {
    let migration_mod_names = vec![PathBuf::from("initial.rs")]
        .into_iter()
        .filter_map(|entry| entry.file_stem().and_then(OsStr::to_str).map(String::from));

    let mut migrations_mods = Vec::new();
    let mut _migrations = Vec::new();

    for migration in migration_mod_names {
        let ident = Ident::new(migration.as_str(), Span2::call_site());
        let mig_mod = quote! {pub mod #ident;};
        _migrations.push(quote! {(#migration, #ident::migration())});
        migrations_mods.push(mig_mod);
    }

    let result = quote! {
        #(#migrations_mods)*

        use refinery::Migration;
        pub fn runner() {
            let quoted_migrations: Vec<(&str, String)> = vec![#(#_migrations),*];
            let migrations = vec![Migration {
                name: "initial".into(),
            }];
        }
    };
    result.into()
}

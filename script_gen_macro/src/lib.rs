extern crate proc_macro;
mod script_component_updater;

use script_component_updater::ScriptComponentUpdaterMacro;
use syn::{self, parse_macro_input, DeriveInput};
use proc_macro::TokenStream;

#[proc_macro_derive(ScriptComponentUpdater, attributes(SyncComponent))]
pub fn derive_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let output = ScriptComponentUpdaterMacro::generate_output(ast);
    TokenStream::from(output)
}

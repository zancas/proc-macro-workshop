extern crate proc_macro;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<DeriveInput>(input.clone()).unwrap();
    let id = ast.ident;
    use quote::{format_ident, quote};
    let builderid = format_ident!("{}Builder", &id);
    quote!(
    impl #id {
        fn builder() -> #builderid {
            #builderid {
                executable: None,
                args: None,
                env: None,
                current_dir: None,
            }
        }
    }

    pub struct #builderid {
        executable: Option<String>,
        args: Option<Vec<String>>,
        env: Option<Vec<String>>,
        current_dir: Option<String>,
    })
    .into()
}

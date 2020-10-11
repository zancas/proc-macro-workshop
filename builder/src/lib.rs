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
    impl #builderid {
        fn executable(&mut self, executable: String) -> &mut Self {
            self.executable = Some(executable);
            self
        }
        fn args(&mut self, args: Vec<String>) -> &mut Self {
            self.args = Some(args);
            self
        }
        fn env(&mut self, env: Vec<String>) -> &mut Self {
            self.env = Some(env);
            self
        }
        fn current_dir(&mut self, current_dir: String) -> &mut Self {
            self.current_dir = Some(current_dir);
            self
        }
        fn build(self) -> Result<#id, Box<dyn Error>> {
            if self.executable.is_none() ||
               self.args.is_none() ||
               self.env.is_none() ||
               self.current_dir.is_none() {
                return Err(Box::new(String::from("foo")));
            };
            Ok(
                #id {
                    executable: self.executable.unwrap(),
                    args: self.args.unwrap(),
                    env: self.env.unwrap(),
                    current_dir: self.current_dir.unwrap(),
            })
        }
    }
    trait Error {}
    impl Error for String {}
    use std::fmt;
    impl std::fmt::Debug for dyn Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("#id").finish()
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

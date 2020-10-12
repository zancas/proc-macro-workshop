extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn _x(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
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
        fn build(&mut self) -> Result<#id, Box<dyn Error>> {
            if self.executable.is_none() ||
               self.args.is_none() ||
               self.env.is_none() ||
               self.current_dir.is_none() {
                return Err(Box::new(String::from("foo")));
            };
            Ok(
                #id {
                    executable: self.executable.as_ref().unwrap().clone(),
                    args: self.args.as_ref().unwrap().clone(),
                    env: self.env.as_ref().unwrap().clone(),
                    current_dir: self.current_dir.as_ref().unwrap().clone(),
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

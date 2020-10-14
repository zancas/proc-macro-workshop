use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use syn::visit_mut::{self, VisitMut};
struct AddOption;
impl VisitMut for AddOption {
    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        if let syn::Type::Path(tp) = &node.ty {
            let syn::TypePath { path, .. } = tp;
            let syn::Path { segments, .. } = path;
            for s in segments {
                if s.ident == "Option" {
                    dbg!("woot");
                }
            }
        }
        visit_mut::visit_field_mut(self, node);
    }
}

#[proc_macro_derive(Builder)]
pub fn xx(input: TokenStream) -> TokenStream {
    // Input section
    //let tokens = input.clone();
    let mut derive_input_ast = parse_macro_input!(input as DeriveInput);
    //dbg!(&derive_input_ast);
    AddOption.visit_derive_input_mut(&mut derive_input_ast);
    let id = derive_input_ast.ident;
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

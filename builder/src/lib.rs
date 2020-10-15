use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use std::vec::Vec;
use syn::visit_mut::{self, VisitMut};

#[derive(Debug)]
pub(crate) struct AddOption {
    pub(crate) mandatory: Vec<String>,
    pub(crate) optional: Vec<String>,
}
impl AddOption {
    pub fn new() -> Self {
        AddOption {
            mandatory: vec![],
            optional: vec![],
        }
    }
}
impl VisitMut for AddOption {
    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        let field_method_name = node.ident.as_ref().unwrap().to_string();
        if let syn::Type::Path(tp) = &node.ty {
            let syn::TypePath { path, .. } = tp;
            let syn::Path { segments, .. } = path;
            let optional = segments.iter().any(|ps| {
                if &ps.ident.to_string() == "Option" {
                    true
                } else {
                    false
                }
            });
            if optional {
                self.optional.push(field_method_name);
            } else {
                self.mandatory.push(field_method_name);
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
    let mut ao = AddOption::new();
    ao.visit_derive_input_mut(&mut derive_input_ast);
    use quote::{format_ident, quote};
    for mandatory_method in ao.mandatory {
        quote!(self.#mandatory_method.is_none());
    }
    let id = derive_input_ast.ident;
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

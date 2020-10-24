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
impl VisitMut for EachElementProcessor {
    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        self.allfields.push(node.clone());
        if !&node.attrs.is_empty() {
            for attr in &node.attrs {
                if attr.path.get_ident().unwrap() == "builder" {
                    let eachfield = node.ident.as_ref().unwrap().clone();
                    if let syn::Meta::List(metalist) = attr.parse_meta().unwrap() {
                        let synmeta = metalist.nested.last().unwrap();
                        //dbg!(&synmeta);
                        match synmeta {
                            syn::NestedMeta::Meta(syn::Meta::NameValue(mnv)) => {
                                self.eachfields.push((eachfield, mnv.lit.clone()));
                                ()
                            }
                            _ => (),
                        }
                    } else {
                        panic!();
                    }
                }
            }
        }
    }
}
#[derive(std::fmt::Debug)]
struct EachElementProcessor {
    allfields: Vec<syn::Field>,
    eachfields: Vec<(syn::Ident, syn::Lit)>,
}
impl EachElementProcessor {
    pub fn new() -> Self {
        EachElementProcessor {
            allfields: vec![],
            eachfields: vec![],
        }
    }
}
#[proc_macro_derive(Builder, attributes(builder))]
pub fn hello_gygaxis(input: TokenStream) -> TokenStream {
    // Input section
    let mut derive_input_ast = parse_macro_input!(input as DeriveInput);
    let mut ao = AddOption::new();
    let mut eep = EachElementProcessor::new();
    eep.visit_derive_input_mut(&mut derive_input_ast);
    ao.visit_derive_input_mut(&mut derive_input_ast);
    dbg!(&eep.allfields[0]);

    for fields in &eep.allfields {
        let methodname = fields.ident.as_ref().unwrap();
        let settertype = fields.ty.clone();
        let method_template = quote!(
        fn #methodname(&mut self, #methodname: #settertype) -> &mut Self {
            self.#methodname = Some(#methodname);
            self
        });
    }
    use quote::{format_ident, quote};
    let setter = &mut String::from("");
    let mandatory_fields = {
        let checker = &mut String::from("if ");
        for required_field in ao.mandatory {
            checker.push_str(&format!("self.{}.is_none() || ", required_field));
            setter.push_str(&format!(
                "{required_field}: self.{required_field}.as_ref().unwrap().clone(),\n",
                required_field = required_field
            ));
        }
        let length = checker.len() - 3;
        checker.truncate(length);
        checker.push_str(r#"{"#);
        checker.push_str(r#"  return Err(Box::new(String::from("foo"))); "#);
        checker.push_str(r#"}"#);
        checker.push_str(r#" else { return Ok(()); }"#);
        checker.clone()
    };
    match ao.optional {
        x if !x.is_empty() => {
            for optional_field in x {
                setter.push_str(&format!(
                    "{optional_field}: self.{optional_field}.clone(),\n",
                    optional_field = optional_field
                ));
            }
        }
        _ => (),
    }
    let mftokens: proc_macro2::TokenStream = mandatory_fields.parse().unwrap();
    let settertokens: proc_macro2::TokenStream = setter.parse().unwrap();
    let id = derive_input_ast.ident;
    let builderid = format_ident!("{}Builder", &id);
    //dbg!(&builderid);
    let methods = quote!(
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
    fn check_mandatory(&self) -> Result<(), Box<dyn Error>>{
        #mftokens
    }
    fn build(&mut self) -> Result<#id, Box<dyn Error>> {
        self.check_mandatory()?;
        Ok(
            #id {
                #settertokens
        })
    }
    );
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
        #methods
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

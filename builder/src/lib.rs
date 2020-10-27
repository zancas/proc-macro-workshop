use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use std::collections::HashMap;
use std::vec::Vec;
use syn::visit_mut::{self, VisitMut};

#[derive(Debug)]
struct BuilderSetterMethodGenerator {
    mandatory: Vec<String>,
    optional: Vec<String>,
}
impl BuilderSetterMethodGenerator {
    fn new() -> Self {
        BuilderSetterMethodGenerator {
            mandatory: vec![],
            optional: vec![],
        }
    }
    fn create_optional_and_mandatory(&mut self, node: &mut syn::Field) {
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
    }
}
impl VisitMut for BuilderSetterMethodGenerator {
    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        self.create_optional_and_mandatory(node);
        visit_mut::visit_field_mut(self, node);
    }
}

struct SetterMethodBuilder {
    settermethods: HashMap<syn::Ident, proc_macro2::TokenStream>,
}
impl SetterMethodBuilder {
    fn new() -> Self {
        SetterMethodBuilder {
            settermethods: HashMap::new(),
        }
    }
}
impl VisitMut for SetterMethodBuilder {
    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        let settermethodname = node.ident.as_ref().unwrap();
        let mut settertype = node.ty.clone();
        if let syn::Type::Path(syn::TypePath { path, .. }) = &settertype {
            let firstsegment = &path.segments.first().unwrap();
            if firstsegment.ident.to_string() == "Option" {
                use syn::PathArguments;
                if let PathArguments::AngleBracketed(abe_args) = &firstsegment.arguments {
                    let inner_type = &abe_args.args.first().unwrap();
                    use syn::GenericArgument;
                    if let GenericArgument::Type(unpacked_type) = inner_type {
                        settertype = unpacked_type.clone();
                    }
                }
            }
        }
        let method_template = quote::quote!(
        fn #settermethodname(&mut self, #settermethodname: #settertype) -> &mut Self {
            self.#settermethodname = Some(#settermethodname);
            self
        });
        self.settermethods
            .insert(settermethodname.clone(), method_template);
        visit_mut::visit_field_mut(self, node);
    }
}
struct EachElementExtender {
    eachfields: Vec<(syn::Ident, syn::Lit)>,
}
impl EachElementExtender {
    fn new() -> Self {
        EachElementExtender { eachfields: vec![] }
    }
}
impl VisitMut for EachElementExtender {
    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        for attr in &node.attrs {
            if attr.path.get_ident().unwrap() == "builder" {
                use syn::Meta::{List, NameValue};
                use syn::NestedMeta::Meta;
                if let List(metalist) = attr.parse_meta().unwrap() {
                    let synmeta = metalist.nested.last().unwrap();
                    if let Meta(NameValue(mnv)) = synmeta {
                        let eachfield = node.ident.as_ref().unwrap().clone();
                        self.eachfields.push((eachfield, mnv.lit.clone()));
                    } else {
                        panic!();
                    }
                } else {
                    panic!();
                }
            }
        }
        visit_mut::visit_field_mut(self, node);
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn hello_gy(input: TokenStream) -> TokenStream {
    // Input section
    let mut derive_input_ast = parse_macro_input!(input as DeriveInput);
    // Apply Visitors
    let mut builder_settermethod_generator = BuilderSetterMethodGenerator::new();
    builder_settermethod_generator.visit_derive_input_mut(&mut derive_input_ast);
    let mut setter_method_builder = SetterMethodBuilder::new();
    setter_method_builder.visit_derive_input_mut(&mut derive_input_ast);
    let mut each_element_extender = EachElementExtender::new();
    each_element_extender.visit_derive_input_mut(&mut derive_input_ast);

    let setter = &mut String::from("");
    let mandatory_fields = {
        let checker = &mut String::from("if ");
        for required_field in builder_settermethod_generator.mandatory {
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
    for optional_field in builder_settermethod_generator.optional {
        setter.push_str(&format!(
            "{optional_field}: self.{optional_field}.clone(),\n",
            optional_field = optional_field
        ));
    }
    let mftokens: proc_macro2::TokenStream = mandatory_fields.parse().unwrap();
    let settertokens: proc_macro2::TokenStream = setter.parse().unwrap();
    let id = derive_input_ast.ident;
    let builderid = quote::format_ident!("{}Builder", &id);
    let mut settermethods: Vec<proc_macro2::TokenStream> = vec![];
    for sm in setter_method_builder.settermethods.values() {
        settermethods.push(sm.clone());
    }
    let methods = quote::quote!(
    #(#settermethods)*
    fn check_mandatory(&self) -> Result<(), Box<dyn Error>>{
        #mftokens
    }
    fn build(&mut self) -> Result<#id, Box<dyn Error>> {
        self.check_mandatory()?;
        Ok(
            #id {
                #settertokens
        })
    });
    quote::quote!(
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

    struct #builderid {
        executable: Option<String>,
        args: Option<Vec<String>>,
        env: Option<Vec<String>>,
        current_dir: Option<String>,
    })
    .into()
}

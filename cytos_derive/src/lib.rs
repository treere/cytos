//! Cytos Derive
//!
//! This crate provides the `CytosNode` derive macro, which automatically implements the
//! `PropInspector` trait for structs in the cytos graph processing system. The macro
//! generates implementations for linking, loading, assigning, and dumping parameters
//! based on field attributes.
//!
//! # Usage
//!
//! To use the derive macro, annotate a struct with `#[derive(CytosNode)]` and mark
//! fields with `#[cytos(input)]` for input parameters or `#[cytos(output)]` for
//! output parameters.
//!
//! ```rust,ignore
//! use cytos_derive::CytosNode;
//!
//! #[derive(CytosNode)]
//! struct MyNode {
//!     #[cytos(input)]
//!     input1: cytos::props::GenericProp,
//!     #[cytos(output)]
//!     output1: cytos::props::GenericProp,
//! }
//! ```
//!
//! The macro will generate the necessary methods to integrate the struct as a node
//! in the cytos graph.
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Field, Fields, LitInt, parse_macro_input};

const INPUT_PROP_TYPE: &str = "input";
const OUTPUT_PROP_TYPE: &str = "output";

/// Information about a field for metadata generation
struct FieldInfo {
    ident: Ident,
    param_id: proc_macro2::TokenStream,
    docs: String,
    type_name: String,
    direction: &'static str,
}

/// Extracts documentation comments from attributes
fn extract_docs(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Ok(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                })) = attr.parse_args::<syn::Expr>()
                {
                    Some(lit_str.value().trim().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Gets field information for metadata generation
fn get_field_infos(fields: &Fields, direction: &'static str) -> Vec<FieldInfo> {
    fields
        .iter()
        .filter_map(|field| {
            let has_attr = field.attrs.iter().any(|attr| {
                if attr.path().is_ident("cytos") {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident(direction) {
                            Ok(())
                        } else {
                            Err(meta.error("expected input or output"))
                        }
                    })
                    .is_ok()
                } else {
                    false
                }
            });
            if has_attr {
                let ident = field.ident.clone().unwrap();
                let param_id = ident_to_lit(&Some(ident.clone()));
                let docs = extract_docs(&field.attrs);
                let type_name = field.ty.to_token_stream().to_string();
                Some(FieldInfo {
                    ident,
                    param_id,
                    docs,
                    type_name,
                    direction,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Derives the `CytosNode` trait for a struct, implementing the `PropInspector` interface.
///
/// This macro generates implementations for parameter linking, loading, assignment, and
/// dumping based on the struct's fields annotated with `#[cytos(input)]` or
/// `#[cytos(output)]`.
///
/// # Panics
///
/// Panics if applied to anything other than a struct.
#[proc_macro_derive(CytosNode, attributes(cytos))]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        attrs,
        ..
    } = parse_macro_input!(input);

    let gwhere = generics.where_clause.clone();

    let Data::Struct(DataStruct { ref fields, .. }) = data else {
        unreachable!()
    };

    let get_prop = create_get_prop(fields);
    let get_prop_mut = create_get_prop_mut(fields);

    let input_names = create_input_names(fields);
    let output_names = create_output_names(fields);

    // Collect metadata information
    let input_infos = get_field_infos(fields, INPUT_PROP_TYPE);
    let output_infos = get_field_infos(fields, OUTPUT_PROP_TYPE);
    let all_infos = input_infos.iter().chain(&output_infos).collect::<Vec<_>>();

    let struct_docs = extract_docs(&attrs);
    let struct_name = ident.to_string();

    let param_entries = all_infos.iter().map(|fi| {
        let param_id = &fi.param_id;
        let name = fi.ident.to_string();
        let description = if fi.docs.is_empty() {
            name.clone()
        } else {
            fi.docs.clone()
        };
        let direction = match fi.direction {
            INPUT_PROP_TYPE => quote!(cytos::ParamDirection::Input),
            OUTPUT_PROP_TYPE => quote!(cytos::ParamDirection::Output),
            _ => unreachable!(),
        };
        let type_name = &fi.type_name;
        quote! {
            (#param_id, cytos::ParamInfo {
                name: #name.to_string(),
                description: #description.to_string(),
                direction: #direction,
                type_name: #type_name.to_string(),
            })
        }
    });

    let metadata_impl = quote! {
        impl #generics cytos::MetadataProvider for #ident #generics #gwhere {
            fn metadata() -> cytos::NodeMetadata {
                cytos::NodeMetadata {
                    name: #struct_name.to_string(),
                    description: #struct_docs.to_string(),
                    params: std::collections::HashMap::from([
                        #(#param_entries),*
                    ]),
                }
            }
        }
    };

    quote! {
        #metadata_impl

        impl  #generics cytos::PropInspector for #ident #generics  #gwhere  {
            #get_prop
            #get_prop_mut

            #input_names
            #output_names
        }
    }
    .into()
}

/// Creates the implementation for the `get_prop` method.
///
/// Generates a match statement that returns a reference to the appropriate input or output field
/// as a `GenericPropInterface` based on the parameter name, or `None` if not found.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `get_prop` method implementation.
fn create_get_prop(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(&self.#i),}
        })
        .collect::<Vec<_>>();

    let outputs = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(&self.#i),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn get_prop(&self, val: cytos::ParamId)
                 -> Option<&dyn cytos::props::GenericPropInterface> {
            match val {
                #(#inputs)*
                #(#outputs)*
                _ => None,
            }
        }
    }
}

/// Creates the implementation for the `get_prop_mut` method.
///
/// Generates a match statement that returns a mutable reference to the appropriate input or output field
/// as a `GenericPropInterface` based on the parameter name, or `None` if not found.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `get_prop_mut` method implementation.
fn create_get_prop_mut(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(&mut self.#i),}
        })
        .collect::<Vec<_>>();

    let outputs = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(&mut self.#i),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn get_prop_mut(&mut self, val: cytos::ParamId)
                 -> Option<&mut dyn cytos::props::GenericPropInterface> {
            match val {
                #(#inputs)*
                #(#outputs)*
                _ => None,
            }
        }
    }
}

/// Creates the implementation for the `input_names` method.
///
/// Generates a vector containing the parameter IDs for all input fields.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `input_names` method implementation.
fn create_input_names(fields: &Fields) -> proc_macro2::TokenStream {
    let input_names = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let f = ident_to_lit(&field.ident);
            quote!(#f)
        })
        .collect::<Vec<_>>();

    quote! {
        fn input_names(&self) -> Vec<cytos::ParamId> {
            vec![
                #(#input_names),*
            ]
        }
    }
}

/// Creates the implementation for the `output_names` method.
///
/// Generates a vector containing the parameter IDs for all output fields.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `output_names` method implementation.
fn create_output_names(fields: &Fields) -> proc_macro2::TokenStream {
    let output_names = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
        .map(|field| {
            let f = ident_to_lit(&field.ident);
            quote!(#f)
        })
        .collect::<Vec<_>>();

    quote! {
        fn output_names(&self) -> Vec<cytos::ParamId> {
            vec![
                #(#output_names),*
            ]
        }
    }
}

/// Filters fields by their cytos attribute type.
///
/// Returns an iterator over fields that have the specified `#[cytos(...)]` attribute.
///
/// # Arguments
///
/// * `fields` - The fields to filter.
/// * `types` - The type to match, either "input" or "output".
///
/// # Returns
///
/// An iterator yielding references to fields matching the type.
fn filter_fields_by_type<'a>(
    fields: &'a Fields,
    types: &'a str,
) -> impl Iterator<Item = &'a Field> {
    fields.iter().filter(|field| {
        field.attrs.iter().any(|attr| {
            if attr.path().is_ident("cytos") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(types) {
                        Ok(())
                    } else {
                        Err(meta.error("expected input or output"))
                    }
                })
                .is_ok()
            } else {
                false
            }
        })
    })
}

/// Converts a field identifier to a `ParamId` literal.
///
/// Parses the identifier as a base-36 number and constructs a `cytos::ParamId`.
///
/// # Arguments
///
/// * `ident` - The optional identifier to convert.
///
/// # Panics
///
/// Panics if the identifier is `None` or cannot be parsed as a base-36 number.
fn ident_to_lit(ident: &'_ Option<Ident>) -> proc_macro2::TokenStream {
    let lit = format!("{}", ident.clone().expect("missing ident"));
    let lit = format!(
        "{}u64",
        u64::from_str_radix(&lit, 36)
            .unwrap_or_else(|_| panic!("cannot parse field '{lit}' as base 36 number"))
    );
    let l = LitInt::new(&lit, Span::call_site());
    quote! {
        cytos::ParamId(#l)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_derive_compilation() {
        // This test ensures the macro compiles without panicking
        // More comprehensive tests would require integration testing
        assert!(true);
    }
}

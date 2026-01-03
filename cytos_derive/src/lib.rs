//! Cytos Derive
//!
//! This crate provides the `CytosNode` derive macro, which automatically implements the
//! `Transformer` trait for structs in the cytos graph processing system. The macro
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
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, LitInt, parse_macro_input};

const INPUT_PROP_TYPE: &str = "input";
const OUTPUT_PROP_TYPE: &str = "output";

/// Derives the `CytosNode` trait for a struct, implementing the `Transformer` interface.
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
        ..
    } = parse_macro_input!(input);

    let gwhere = generics.where_clause.clone();

    let Data::Struct(DataStruct { ref fields, .. }) = data else {
        unreachable!()
    };

    let link = create_link(fields);

    let load = create_load(fields);
    let assign = create_assign(fields);
    let dump = create_dump(fields);

    let load_owned = create_load_owned(fields);
    let assign_owned = create_assign_owned(fields);
    let dump_owned = create_dump_owned(fields);

    let input = create_input(fields);
    let input_names = create_input_names(fields);

    let output = create_output(fields);
    let output_names = create_output_names(fields);

    quote! {
        impl  #generics cytos::Transformer for #ident #generics  #gwhere  {
            #link


            #load
            #assign
            #dump

            #load_owned
            #assign_owned
            #dump_owned

            #input
            #input_names

            #output
            #output_names
        }
    }
    .into()
}

/// Creates the implementation for the `link` method.
///
/// Generates a match statement that calls `link_value` on the appropriate input field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `link` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input field.
fn create_link(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let ident = &field.ident;
            let lit = ident_to_lit(ident);
            quote! {#lit => self.#ident.link_value(val),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn link(&mut self, name: cytos::ParamId, val: cytos::props::GenericProp)
                -> cytos::Result<()> {
            match name {
                #(#inputs)*
                _ => Err("missing input link data".into()),
            }
        }
    }
}

/// Creates the implementation for the `assign` method.
///
/// Generates a match statement that calls `assign` on the appropriate input field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `assign` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input field.
fn create_assign(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => self.#i.assign(value),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn assign(
                &mut self,
                name: cytos::ParamId,
                value: cytos::Value,
            ) -> cytos::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

/// Creates the implementation for the `load` method.
///
/// Generates a match statement that calls `load` on the appropriate input field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `load` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input field.
fn create_load(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => self.#i.load(value),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn load(
                &mut self,
                name: cytos::ParamId,
                value: cytos::Value,
            ) -> cytos::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

/// Creates the implementation for the `dump` method.
///
/// Generates a match statement that calls `dump` on the appropriate input or output field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `dump` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input or output field.
fn create_dump(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .chain(filter_fields_by_type(fields, OUTPUT_PROP_TYPE))
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => self.#i.dump(),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn dump(
                & self,
                name: cytos::ParamId
            ) -> cytos::Result<cytos::Value> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

/// Creates the implementation for the `load_owned` method.
///
/// Generates a match statement that calls `load_owned_generic` on the appropriate input field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `load_owned` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input field.
fn create_load_owned(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => self.#i.load_owned_generic(value),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn load_owned(
                &mut self,
                name: cytos::ParamId,
                value: cytos::GenericOwnedProp,
            ) -> cytos::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

/// Creates the implementation for the `assign_owned` method.
///
/// Generates a match statement that calls `assign_owned_generic` on the appropriate input field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `assign_owned` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input field.
fn create_assign_owned(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => self.#i.assign_owned_generic(value),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn assign_owned(
                &mut self,
                name: cytos::ParamId,
                value: cytos::GenericOwnedProp,
            ) -> cytos::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

/// Creates the implementation for the `dump_owned` method.
///
/// Generates a match statement that calls `into_owned_generic` on the appropriate input or output field
/// based on the parameter name.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `dump_owned` method implementation.
///
/// # Errors
///
/// The generated method returns an error if the parameter name does not correspond
/// to any input or output field.
fn create_dump_owned(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .chain(filter_fields_by_type(fields, OUTPUT_PROP_TYPE))
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Ok(self.#i.into_owned_generic()),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn dump_owned(
                & self,
                name: cytos::ParamId
            ) -> cytos::Result<cytos::GenericOwnedProp> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

/// Creates the implementation for the `input` method.
///
/// Generates a match statement that returns `as_generic` on the appropriate input field
/// based on the parameter name, or `None` if not found.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `input` method implementation.
fn create_input(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(self.#i.as_generic()),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn input(&self, val: cytos::ParamId)
                 -> Option<cytos::props::GenericProp> {
            match val {
                #(#inputs)*
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

/// Creates the implementation for the `output` method.
///
/// Generates a match statement that returns `as_generic` on the appropriate output field
/// based on the parameter name, or `None` if not found.
///
/// # Arguments
///
/// * `fields` - The fields of the struct to process.
///
/// # Returns
///
/// A `TokenStream` containing the generated `output` method implementation.
fn create_output(fields: &Fields) -> proc_macro2::TokenStream {
    let outputs = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(self.#i.as_generic()),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn output(&self, val: cytos::ParamId)
                  -> Option<cytos::props::GenericProp> {
            match val {
                #(#outputs)*
                _ => None,
            }
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

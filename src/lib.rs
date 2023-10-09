use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, LitStr, TypePath};

#[proc_macro_derive(TransFn)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let fields = if let Data::Struct(DataStruct { ref fields, .. }) = data {
        fields
    } else {
        unreachable!();
    };

    let input_names = create_input_names(fields);
    let output_names = create_output_names(fields);
    let output = create_output(fields);
    let input = create_input(fields);
    let link = create_link(fields);

    quote! {
        impl proph::architecture::Transformer for #ident {
            #link

            #output

            #input

            #input_names

            #output_names
        }

    }
    .into()
}

fn create_link(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = fields
        .iter()
        .filter(|field| match &field.ty {
            syn::Type::Path(TypePath { path, .. }) => {
                let s = path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join(":");
                ["InputProp"].contains(&s.as_str())
            }
            _ => true,
        })
        .map(|field| {
            let i = &field.ident;
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote! {#f => self.#i.change_value(val),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn link(&mut self, name: proph::architecture::ParamId, val: proph::architecture::GenericOutputProp)
                -> Result<(), &'static str> {
            match name.as_str() {
                #(#inputs)*
                _ => Err("missing"),
            }
        }
    }
}

fn create_input(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = fields
        .iter()
        .filter(|field| match &field.ty {
            syn::Type::Path(TypePath { path, .. }) => {
                let s = path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join(":");
                ["InputProp"].contains(&s.as_str())
            }
            _ => true,
        })
        .map(|field| {
            let i = &field.ident;
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote! {#f => Some(self.#i.as_generic()),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn input(&self, val: proph::architecture::ParamId)
                 -> Option<proph::architecture::GenericInputProp> {
            match val.as_str() {
                #(#inputs)*
                _ => None,
            }
        }
    }
}

fn create_input_names(fields: &Fields) -> proc_macro2::TokenStream {
    let input_names = fields
        .iter()
        .filter(|field| match &field.ty {
            syn::Type::Path(TypePath { path, .. }) => {
                let s = path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join(":");
                ["InputProp"].contains(&s.as_str())
            }
            _ => true,
        })
        .map(|field| {
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote!(#f.to_owned())
        })
        .collect::<Vec<_>>();

    quote! {
        fn input_names(&self) -> Vec<proph::architecture::ParamId> {
            vec![
                #(#input_names),*
            ]
        }
    }
}

fn create_output(fields: &Fields) -> proc_macro2::TokenStream {
    let outputs = fields
        .iter()
        .filter(|field| match &field.ty {
            syn::Type::Path(TypePath { path, .. }) => {
                let s = path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join(":");
                ["OutputProp"].contains(&s.as_str())
            }
            _ => true,
        })
        .map(|field| {
            let i = &field.ident;
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote! {#f => Some(self.#i.as_generic()),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn output(&self, val: proph::architecture::ParamId)
                  -> Option<proph::architecture::GenericOutputProp> {
            match val.as_str() {
                #(#outputs)*
                _ => None,
            }
        }
    }
}

fn create_output_names(fields: &Fields) -> proc_macro2::TokenStream {
    let output_names = fields
        .iter()
        .filter(|field| match &field.ty {
            syn::Type::Path(TypePath { path, .. }) => {
                let s = path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join(":");
                ["OutputProp"].contains(&s.as_str())
            }
            _ => true,
        })
        .map(|field| {
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote!(#f.to_owned())
        })
        .collect::<Vec<_>>();

    quote! {
        fn output_names(&self) -> Vec<proph::architecture::ParamId> {
            vec![
                #(#output_names),*
            ]
        }
    }
}

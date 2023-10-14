use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, LitStr, Type, TypePath,
};

const INPUT_PROP_TYPE: &[&str] = &["InputProp"];
const OUTPUT_PROP_TYPE: &[&str] = &["OutputProp"];

#[proc_macro_derive(TransFn)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let fields = if let Data::Struct(DataStruct { ref fields, .. }) = data {
        fields
    } else {
        unreachable!();
    };

    let link = create_link(fields);
    let load = create_load(fields);
    let dump = create_dump(fields);

    let input = create_input(fields);
    let input_names = create_input_names(fields);

    let output = create_output(fields);
    let output_names = create_output_names(fields);

    quote! {
        impl proph::architecture::Transformer for #ident {
            #link
            #load
            #dump

            #input
            #input_names

            #output
            #output_names
        }
    }
    .into()
}

fn create_link(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let ident = &field.ident;
            let lit = LitStr::new(
                &format!("{}", ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote! {#lit => self.#ident.change_value(val),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn link(&mut self, name: proph::architecture::ParamId, val: proph::architecture::props::GenericOutputProp)
                -> proph::architecture::Done {
            match name.as_str() {
                #(#inputs)*
                _ => Err("missing input link data"),
            }
        }
    }
}

fn create_load(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote! {#f => self.#i.load(value),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn load(
                &mut self,
                name: proph::architecture::ParamId,
                value: &str,
            ) -> proph::architecture::Done {
                match name.as_str() {
                    #(#inputs)*
                    _ => Err("parameter not found"),
                }

            }
        )
    }
}

fn create_dump(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = LitStr::new(
                &format!("{}", field.ident.clone().expect("missing ident")),
                Span::call_site(),
            );
            quote! {#f => self.#i.dump(),}
        })
        .collect::<Vec<_>>();

    {
        quote!(
            fn dump(
                & self,
                name: proph::architecture::ParamId
            ) -> Result<String, &'static str> {
                match name.as_str() {
                    #(#inputs)*
                    _ => Err("parameter not found"),
                }

            }
        )
    }
}

fn create_input(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
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
                 -> Option<proph::architecture::props::GenericInputProp> {
            match val.as_str() {
                #(#inputs)*
                _ => None,
            }
        }
    }
}

fn create_input_names(fields: &Fields) -> proc_macro2::TokenStream {
    let input_names = filter_fields_by_type(fields, INPUT_PROP_TYPE)
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
    let outputs = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
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
                  -> Option<proph::architecture::props::GenericOutputProp> {
            match val.as_str() {
                #(#outputs)*
                _ => None,
            }
        }
    }
}

fn create_output_names(fields: &Fields) -> proc_macro2::TokenStream {
    let output_names = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
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

fn filter_fields_by_type<'a>(
    fields: &'a Fields,
    types: &'a [&'_ str],
) -> impl Iterator<Item = &'a Field> {
    fields.iter().filter(|field| is_of_type(&field.ty, types))
}

fn is_of_type(ty: &Type, types: &[&str]) -> bool {
    match ty {
        syn::Type::Path(TypePath { path, .. }) => {
            let s = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join(":");
            types.contains(&s.as_str())
        }
        _ => false,
    }
}

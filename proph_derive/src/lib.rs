use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, LitInt};

const INPUT_PROP_TYPE: &str = "input";
const OUTPUT_PROP_TYPE: &str = "output";

#[proc_macro_derive(ProphNode, attributes(input, output))]
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
        impl  #generics proph::architecture::Transformer for #ident #generics  #gwhere  {
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

fn create_link(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let ident = &field.ident;
            let lit = ident_to_lit(ident);
            quote! {#lit => self.#ident.link_value(val),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn link(&mut self, name: proph::architecture::ParamId, val: proph::architecture::props::GenericProp)
                -> proph::architecture::Result<()> {
            match name {
                #(#inputs)*
                _ => Err("missing input link data".into()),
            }
        }
    }
}

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
                name: proph::architecture::ParamId,
                value: proph::architecture::Value,
            ) -> proph::architecture::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

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
                name: proph::architecture::ParamId,
                value: proph::architecture::Value,
            ) -> proph::architecture::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

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
                name: proph::architecture::ParamId
            ) -> proph::architecture::Result<proph::architecture::Value> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

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
                name: proph::architecture::ParamId,
                value: proph::architecture::GenericOwnedProp,
            ) -> proph::architecture::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

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
                name: proph::architecture::ParamId,
                value: proph::architecture::GenericOwnedProp,
            ) -> proph::architecture::Result<()> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

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
                name: proph::architecture::ParamId
            ) -> proph::architecture::Result<proph::architecture::GenericOwnedProp> {
                match name {
                    #(#inputs)*
                    _ => Err("parameter not found".into()),
                }

            }
        )
    }
}

fn create_input(fields: &Fields) -> proc_macro2::TokenStream {
    let inputs = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let i = &field.ident;
            let f = ident_to_lit(i);
            quote! {#f => Some(self.#i.as_generic()),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn input(&self, val: proph::architecture::ParamId)
                 -> Option<proph::architecture::props::GenericProp> {
            match val {
                #(#inputs)*
                _ => None,
            }
        }
    }
}

fn create_input_names(fields: &Fields) -> proc_macro2::TokenStream {
    let input_names = filter_fields_by_type(fields, INPUT_PROP_TYPE)
        .map(|field| {
            let f = ident_to_lit(&field.ident);
            quote!(#f)
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
            let f = ident_to_lit(i);
            quote! {#f => Some(self.#i.as_generic()),}
        })
        .collect::<Vec<_>>();

    quote! {
        fn output(&self, val: proph::architecture::ParamId)
                  -> Option<proph::architecture::props::GenericProp> {
            match val {
                #(#outputs)*
                _ => None,
            }
        }
    }
}

fn create_output_names(fields: &Fields) -> proc_macro2::TokenStream {
    let output_names = filter_fields_by_type(fields, OUTPUT_PROP_TYPE)
        .map(|field| {
            let f = ident_to_lit(&field.ident);
            quote!(#f)
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
    types: &'a str,
) -> impl Iterator<Item = &'a Field> {
    fields
        .iter()
        .filter(|field| field.attrs.iter().any(|attr| attr.path().is_ident(types)))
}

fn ident_to_lit(ident: &'_ Option<Ident>) -> proc_macro2::TokenStream {
    let lit = format!("{}", ident.clone().expect("missing ident"));
    let lit = format!(
        "{}u64",
        u64::from_str_radix(&lit, 36).expect("cannot parse")
    );
    let l = LitInt::new(&lit, Span::call_site());
    quote! {
        proph::architecture::ParamId(#l)
    }
}

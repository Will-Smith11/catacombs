use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse::Parser, punctuated::Punctuated, token::Comma, Field, Fields, ItemStruct};

fn basic_name(input: &Ident) -> Ident
{
    let mut channel_type = input.to_string().to_lowercase();
    channel_type.truncate(channel_type.len() - 2);
    Ident::new(&channel_type, input.span())
}

fn is_tx(input: &Ident) -> bool
{
    input.to_string().to_lowercase().ends_with("tx")
}

fn build_name(input: &Ident) -> Ident
{
    let mut channel_type = input.to_string().to_lowercase();
    channel_type.truncate(channel_type.len() - 2);
    if is_tx(input)
    {
        channel_type.push_str("_tx");
    }
    else
    {
        channel_type.push_str("_rx");
    }
    Ident::new(&channel_type, input.span())
}

fn build_fields(input: Vec<&Ident>) -> Punctuated<Field, Comma>
{
    input
        .into_iter()
        .map(|x| {
            let field_name = basic_name(&x);
            Field {
                attrs:       vec![],
                vis:         syn::Visibility::Inherited,
                ident:       Some(field_name),
                colon_token: Some(syn::token::Colon::default()),
                ty:          syn::parse2(quote! { #x }).unwrap()
            }
        })
        .collect()
}

fn create_injetion_code(i_struct: &ItemStruct, channels: Vec<&Ident>) -> TokenStream
{
    let struct_name = &i_struct.ident;
    let struct_generics = &i_struct.generics;

    let calls = channels.iter().map(|x| {
        let ident = build_name(x);
        quote!( #ident() )
    });

    quote!(
        impl #struct_generics #struct_name #struct_generics {
            pub fn get_channels() -> (#(#channels),*) {
                // need to build singleton before we can go farther
                let catacomb = get_catacomb();
                let mut lock = catacomb.lock().as_mut().unwrap();
                (
                    #(
                        lock.#calls
                    ),*
                )
            }
        }
    )
}

pub fn parse_struct(args: TokenStream, input: TokenStream) -> TokenStream
{
    let mut item_struct: ItemStruct = syn::parse2(input.into()).unwrap();

    let args_parsed = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
        .parse(args.into())
        .unwrap();

    let meta = args_parsed
        .iter()
        .map(|x| x.get_ident().unwrap())
        .collect::<Vec<_>>();
    let injection_code = create_injetion_code(&item_struct, meta.clone());
    let new_fields = build_fields(meta);
    if let Fields::Named(fields) = &mut item_struct.fields
    {
        fields.named.extend(new_fields);
    }

    quote! {
        #item_struct
        #injection_code
    }
}

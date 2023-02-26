use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    ItemImpl, ItemStruct
};

enum Direction
{
    Sender,
    Receiver
}

fn parse_direction(input: &Ident) -> Direction
{
    if input.to_string().to_lowercase().ends_with("tx")
    {
        Direction::Sender
    }
    else if input.to_string().to_lowercase().ends_with("rx")
    {
        Direction::Receiver
    }
    else
    {
        panic!("Channel name must end with either 'tx' or 'rx'")
    }
}

fn parse_channel_type(input: &Ident) -> Ident
{
    let mut channel_type = input.to_string();
    channel_type.truncate(channel_type.len());
    Ident::new(&channel_type, input.span())
}

pub fn parse_struct(args: TokenStream, input: TokenStream) -> TokenStream
{
    let item_struct: ItemStruct = syn::parse2(input.into()).unwrap();

    let args_parsed = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
        .parse(args.into())
        .unwrap();

    let meta = args_parsed
        .iter()
        .map(|x| x.get_ident().unwrap())
        .collect::<Vec<_>>();

    quote! {
        #item_struct
    }
}

pub fn parse_inject(args: TokenStream, input: TokenStream) -> TokenStream
{
    let item_struct: ItemImpl = syn::parse2(input.into()).unwrap();

    let args_parsed = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
        .parse(args.into())
        .unwrap();

    let meta = args_parsed
        .iter()
        .map(|x| x.get_ident().unwrap())
        .collect::<Vec<_>>();

    quote! {
        #item_struct
    }
}

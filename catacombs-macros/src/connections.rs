use std::{
    fmt::{Display, Formatter},
    str::FromStr
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitInt, Token
};

pub enum ChannelType
{
    Mpsc,
    Mpmc
}

impl Display for ChannelType
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            ChannelType::Mpsc => write!(f, "tokio::sync::mpsc"),
            ChannelType::Mpmc => write!(f, "tokio::sync::broadcast")
        }
    }
}
impl FromStr for ChannelType
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s.to_lowercase().as_str()
        {
            "mpsc" => Ok(Self::Mpsc),
            "mpmc" => Ok(Self::Mpmc),
            _ => Err(format!("Unknown channel type: {}", s))
        }
    }
}

fn type_def(data: &DataRow) -> TokenStream
{
    let sender = Ident::new(&format!("{}Tx", data.data_pasing), Span::call_site());
    let receiver = Ident::new(&format!("{}Rx", data.data_pasing), Span::call_site());
    let dtype = Ident::new(&data.data_pasing, Span::call_site());

    let c_type = TokenStream::from_str(&data.channel_type.to_string()).unwrap();

    quote!(
        type #sender = #c_type::Sender<#dtype>;
        type #receiver = #c_type::Receiver<#dtype>;
    )
}

fn channel_fn(data: &DataRow) -> TokenStream
{
    let sender = Ident::new(&format!("{}Tx", data.data_pasing), Span::call_site());
    let receiver = Ident::new(&format!("{}Rx", data.data_pasing), Span::call_site());

    match data.channel_type
    {
        ChannelType::Mpsc =>
        {
            quote!(
                pub fn get_receiver(&mut self) -> #receiver {
                    self.receiver.take().unwrap()
                }

                pub fn get_sender(&mut self) -> #sender {
                    self.sender.clone()
                }
            )
        }
        ChannelType::Mpmc =>
        {
            quote!(
                pub fn get_receiver(&mut self) -> #receiver {
                    self.sender.subscribe()
                }

                pub fn get_sender(&mut self) -> #sender {
                    self.sender.clone()
                }
            )
        }
    }
}

fn create_holder_struct(name: &Ident, data: &DataRow) -> TokenStream
{
    let sender = Ident::new(&format!("{}Tx", data.data_pasing), Span::call_site());
    let receiver = Ident::new(&format!("{}Rx", data.data_pasing), Span::call_site());
    let c_type = TokenStream::from_str(&data.channel_type.to_string()).unwrap();
    let buffer = data.buffer_size;
    let channel_fn = channel_fn(data);

    quote!(
        pub struct #name
        {
            pub sender: #sender,
            pub receiver: Option<#receiver>
        }

        impl #name
        {
            pub fn new() -> Self
            {
                let (sender, receiver) = #c_type::channel(#buffer);
                Self { sender, receiver: Some(receiver) }
            }

            #channel_fn
        }
    )
}

fn build_singleton(rows: &Vec<DataRow>) -> TokenStream
{
    let lowers = rows
        .iter()
        .map(|row| Ident::new(&row.data_pasing.to_lowercase(), Span::call_site()))
        .collect::<Vec<_>>();

    let types = rows
        .iter()
        .map(|row| Ident::new(&format!("{}Inner", row.data_pasing), Span::call_site()))
        .collect::<Vec<_>>();

    let tx_name = rows
        .iter()
        .map(|row| Ident::new(&format!("{}_tx", row.data_pasing.to_lowercase()), Span::call_site()))
        .collect::<Vec<_>>();
    let rx_name = rows
        .iter()
        .map(|row| Ident::new(&format!("{}_rx", row.data_pasing.to_lowercase()), Span::call_site()))
        .collect::<Vec<_>>();

    let tx = rows
        .iter()
        .map(|row| Ident::new(&format!("{}Tx", row.data_pasing), Span::call_site()));
    let rx = rows
        .iter()
        .map(|row| Ident::new(&format!("{}Rx", row.data_pasing), Span::call_site()));

    quote!(
        pub static mut SINGLETON:
            parking_lot::Mutex<Option<CatacombSingleton>> = parking_lot::Mutex::new(None);

        pub fn init_catacomb()
        {
            unsafe {
                SINGLETON = parking_lot::Mutex::new(Some(CatacombSingleton::new()));
            }
        }

        pub fn get_catacomb() -> &'static CatacombSingleton
        {
            unsafe {
                SINGLETON.lock().as_ref().unwrap()
            }
        }

        pub struct CatacombSingleton {
            #(#lowers: once_cell::sync::Lazy<#types>),*
        }

        impl CatacombSingleton {
            pub fn new() -> Self {
                Self {
                    #(#lowers: once_cell::sync::Lazy::new(|| #types::new())),*
                }
            }

            #(
                pub fn #tx_name(&self) -> #tx{
                    self.#lowers.get_sender()
                }
                pub fn #rx_name(&self) -> #rx {
                    self.#lowers.get_receiver()
                }
            )*


        }

    )
}

/// this is the hard part as we need to create a singleton that can be used to
/// fetch all the channels.
///  we also need to typedef all the given channels so that we can properly
/// generate the types for our injection  as well as
pub fn parse_connections(tokens: TokenStream) -> TokenStream
{
    let Data { data } = syn::parse2(tokens).unwrap();

    let inner_holders = data.iter().map(|d| {
        let name = Ident::new(&format!("{}Inner", d.data_pasing), Span::call_site());

        create_holder_struct(&name, d)
    });

    let types = data.iter().map(type_def).collect::<Vec<_>>();
    let singleton = build_singleton(&data);

    quote! {
        #(#types)*
        #(#inner_holders)*
        #singleton
    }
}

struct Data
{
    data: Vec<DataRow>
}

impl Parse for Data
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let mut rows = Vec::new();
        while !input.is_empty()
        {
            rows.push(input.parse::<DataRow>()?);
        }

        Ok(Self { data: rows })
    }
}

struct DataRow
{
    data_pasing:  String,
    channel_type: ChannelType,
    buffer_size:  usize
}

impl Parse for DataRow
{
    fn parse(input: ParseStream<'_>) -> syn::Result<Self>
    {
        let data_pasing = input
            .parse::<Ident>()
            .map_err(|_| syn::Error::new(input.span(), "failed to parse transport type"))?
            .to_string();

        if !input.peek(Token![,])
        {
            return Err(syn::Error::new(input.span(), "expected `,`"))
        }

        let _ = input.parse::<Token![,]>()?;
        let channel_type = ChannelType::from_str(
            &input
                .parse::<Ident>()
                .map_err(|e| {
                    syn::Error::new(input.span(), format!("{e}, failed to parse channel type"))
                })?
                .to_string()
        )
        .map_err(|_| syn::Error::new(input.span(), "invalid channel type"))?;

        if !input.peek(Token![,])
        {
            return Err(syn::Error::new(input.span(), "expected `,`"))
        }
        let _ = input.parse::<Token![,]>()?;

        let buffer_size = usize::from_str(
            &input
                .parse::<LitInt>()
                .map_err(|_| syn::Error::new(input.span(), "failed to parse buffer_size"))?
                .to_string()
        )
        .map_err(|_| syn::Error::new(input.span(), "invalid buffer size"))?;

        if !input.peek(Token![;])
        {
            return Err(syn::Error::new(input.span(), "expected `;`"))
        }
        let _ = input.parse::<Token![;]>()?;

        Ok(Self { data_pasing, channel_type, buffer_size })
    }
}

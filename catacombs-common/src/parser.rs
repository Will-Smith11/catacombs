//! The parser helps us deal with runtime injections as well as allow as moving
//! runtime errors into compile time errors. We do this by having our top level
//! build.rs file grab the definitions of the channels and then also fetch all
//! implementations of such channels. we can use this context to generate the
//! injection code for the user. we then use the #[inject] macro to simply
//! delete the function after we have already copied the function and added the
//! injection code. to start off with, we will only be injecting the init
//! function into the user's code. In the future we will add compile time checks
//! on channel fetches

use std::{
    collections::HashMap,
    path::{Path, PathBuf}
};

use proc_macro2::TokenStream;
use rayon::prelude::*;
use syn::{parse::Parse, Ident, ImplItemMethod, ItemImpl, ItemStruct};

const CHANNEL_TAG: &str = "#[channel(";
const INJECT_TAG: &str = "#[inject]";

pub fn init_catacombs(root: &Path)
{
    let all_files = filter_files_by_tag(get_all_file_paths(root));

    let parsed_files = all_files
        .into_iter()
        .map(|file| {
            let content = std::fs::read_to_string(&file).unwrap();
            let input: TokenStream = content.parse().unwrap();
            let item_struct: ParsedFile = syn::parse2(input.into()).unwrap();
            (file, item_struct)
        })
        .collect::<Vec<(PathBuf, ParsedFile)>>();

    // next thing we want todo is to link the structs and impls together along with
    // there file path. this is so we can inject the code into the correct
    // location

    // get matching impl and struct cases
    let mut structs = HashMap::new();
    let mut implmentations = HashMap::new();

    for (_, parsed_file) in parsed_files
    {
        for impls in parsed_file.impls
        {
            if impls.trait_.is_some()
            {
                panic!("traits are not supported");
            }

            let struct_name = match *impls.clone().self_ty
            {
                syn::Type::Path(path) => path.path.segments.last().unwrap().ident.clone(),
                _ => panic!("unsupported type")
            };

            implmentations
                .entry(struct_name)
                .or_insert(Vec::new())
                .push(impls);
        }

        for stru in parsed_file.structs
        {
            structs.insert(stru.ident.clone(), stru);
        }
    }

    // only look at struts with the channel attr
    structs.retain(|_, stru| stru.attrs.iter().any(|attr| attr.path.is_ident("channel")));

    // only look at impl blocks that have the inject attr on a function
    implmentations.retain(|_, v| {
        v.iter().any(|impls| {
            impls.items.iter().any(|item| match { item }
            {
                syn::ImplItem::Method(method) =>
                {
                    method.attrs.iter().any(|attr| attr.path.is_ident("inject"))
                }
                _ => false
            })
        })
    });

    // now lets grab the all the struts and then build our init injection code
    // pure pain
    for (ident, st) in structs
    {
        if let Some(impld) = implmentations.get(&ident)
        {
            'a: for impls in impld
            {
                for implx in &impls.items
                {
                    match implx
                    {
                        syn::ImplItem::Method(meth) =>
                        {
                            if meth.attrs.iter().any(|attr| attr.path.is_ident("inject"))
                            {
                                inject_fn(&ident, meth, st);
                                break 'a
                            }
                        }
                        _ => continue
                    }
                }
            }
        }
    }
}

/// Our Channel Macro handles the inclusion of what we are building here.
pub fn inject_fn(name: &Ident, meth: &ImplItemMethod, stru: ItemStruct)
{
    // first thing we want todo is to find the creation of the Self struct.
    // from there we need to build out the statement of the code.
    // we do this by searching for the Expression struct and then match that
    // against, our name ident. we then know that we need to inject the code
    // from there .
    meth.block.stmts.iter().for_each(|stmt| match stmt
    {
        syn::Stmt::Local(l) =>
        {}
        syn::Stmt::Item(i) =>
        {}
        syn::Stmt::Semi(v, _) =>
        {}
        syn::Stmt::Expr(e) =>
        {}
    });
}

pub struct ParsedFile
{
    pub structs: Vec<ItemStruct>,
    pub impls:   Vec<ItemImpl>
}

impl Parse for ParsedFile
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let mut structs = Vec::new();
        let mut impls = Vec::new();

        while !input.is_empty()
        {
            let item: syn::Item = input.parse()?;
            match item
            {
                syn::Item::Struct(s) => structs.push(s),
                syn::Item::Impl(i) => impls.push(i),
                _ =>
                {}
            }
        }
        Ok(Self { structs, impls })
    }
}

fn filter_files_by_tag(files: Vec<PathBuf>) -> Vec<PathBuf>
{
    files
        .into_par_iter()
        .filter_map(|file| {
            let content = std::fs::read_to_string(&file).unwrap();
            if content.contains(CHANNEL_TAG) || content.contains(INJECT_TAG)
            {
                Some(file)
            }
            else
            {
                None
            }
        })
        .collect()
}

fn get_all_file_paths(root: &Path) -> Vec<PathBuf>
{
    let mut res = Vec::new();
    let mut new_paths = vec![root.to_path_buf()];
    // TODO: see if we can parallelize in any way.
    loop
    {
        let Some(path) = new_paths.pop() else { break; };
        let Ok(dirs) = path.read_dir() else { continue; };

        for entry in dirs
        {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir()
            {
                new_paths.push(path);
            }
            else
            {
                res.push(path);
            }
        }
        if new_paths.is_empty()
        {
            break
        }
    }

    res
}

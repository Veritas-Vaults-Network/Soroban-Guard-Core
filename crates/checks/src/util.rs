//! Shared helpers for walking `#[contractimpl]` impl blocks.

use syn::{ImplItem, Item, ItemImpl};

pub fn is_contractimpl(item_impl: &ItemImpl) -> bool {
    item_impl
        .attrs
        .iter()
        .any(|attr| path_is_contractimpl(attr.path()))
}

fn path_is_contractimpl(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|s| s.ident == "contractimpl")
}

/// Every function item inside a `#[contractimpl]` impl in the file.
pub fn contractimpl_functions(file: &syn::File) -> Vec<&syn::ImplItemFn> {
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };
        if !is_contractimpl(item_impl) {
            continue;
        }
        for impl_item in &item_impl.items {
            if let ImplItem::Fn(m) = impl_item {
                out.push(m);
            }
        }
    }
    out
}

/// Name bound by a `let` pattern, seeing through an explicit type annotation so
/// `let admin: Address = ..` resolves to `admin` just like `let admin = ..`.
pub fn binding_ident(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::Ident(p) => Some(p.ident.to_string()),
        syn::Pat::Type(p) => binding_ident(&p.pat),
        _ => None,
    }
}

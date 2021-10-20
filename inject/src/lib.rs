extern crate proc_macro;
use proc_macro::*;
use quote::quote;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn random_panic(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item: syn::Item = syn::parse(input).unwrap();
  let fn_item = match &mut item {
    syn::Item::Fn(fn_item) => fn_item,
    _ => panic!("This attribute only targets function"),
  };
  let statements = &mut fn_item.block.stmts;
  let len = statements.len();
  let at = rand::random::<usize>() % len;
  statements.insert(at, syn::parse(quote!(crate::panic::random_panic();).into()).unwrap());

  item.into_token_stream().into()
}

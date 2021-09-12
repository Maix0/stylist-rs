use proc_macro2::{TokenStream, TokenTree};

use std::collections::{HashMap, HashSet};

use litrs::StringLit;
use proc_macro_error::{abort, abort_call_site};
use std::convert::TryFrom;

use stylist_core::ast::Sheet;

pub mod argument;
mod fstring;
mod to_output_with_args;

use argument::Argument;
use to_output_with_args::ToOutputWithArgs;

use crate::output::{Reify, ReifyContext};

pub(crate) fn macro_fn(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();

    let first_token = match tokens.next() {
        Some(m) => m,
        None => abort_call_site!("expected at least one argument"),
    };

    let s_literal = match StringLit::try_from(first_token.clone()) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error2(),
    };

    let sheet: Sheet = match s_literal.value().parse() {
        Ok(m) => m,

        Err(e) => abort!(first_token, "{}", e.to_string()),
    };

    let mut args = HashMap::new();

    let is_comma = |t: &TokenTree| -> bool {
        match t {
            TokenTree::Punct(m) => m.as_char() == ',',
            _ => false,
        }
    };

    let is_equal = |t: &TokenTree| -> bool {
        match t {
            TokenTree::Punct(m) => m.as_char() == '=',
            _ => false,
        }
    };

    let mut comma_read = false;

    'outer: loop {
        if !comma_read {
            match tokens.next() {
                Some(m) => {
                    if !is_comma(&m) {
                        abort!(m, "expected ',', got: {}", m)
                    }
                }
                None => break 'outer,
            };
        }

        let name_token = match tokens.next() {
            Some(m) => m,
            None => break 'outer,
        };

        let name_ident = match name_token {
            TokenTree::Ident(ref m) => m,
            _ => abort!(name_token, "expected ident, got: {}", name_token),
        };

        let name = name_ident.to_string();

        let mut arg = Argument {
            name,
            name_token: name_ident.clone(),
            tokens: TokenStream::new(),
        };

        if !tokens.next().map(|m| is_equal(&m)).unwrap_or(false) {
            abort!(
                name_token,
                "expected = at the end of this ident, only named arguments are allowed at this moment";
                hint = format!("try: {name} = {name}", name = arg.name),
            );
        }

        'inner: loop {
            let next_token = match tokens.next() {
                Some(m) => m,
                None => {
                    if args.insert(arg.name.clone(), arg).is_some() {
                        abort!(name_token, "duplicate named argument");
                    }
                    break 'outer;
                }
            };

            if is_comma(&next_token) {
                if args.insert(arg.name.clone(), arg).is_some() {
                    abort!(name_token, "duplicate named argument");
                }
                comma_read = true;
                break 'inner;
            }

            arg.tokens.extend(TokenStream::from(next_token));
        }
    }

    let mut args_used = HashSet::with_capacity(args.len());

    let output = sheet.to_output_with_args(&args, &mut args_used);

    for (k, v) in args.iter() {
        if !args_used.contains(k) {
            abort!(
                v.name_token,
                "argument {} is not used, arguments must be used",
                k
            );
        }
    }

    let mut ctx = ReifyContext::new();
    output.into_token_stream(&mut ctx)
}

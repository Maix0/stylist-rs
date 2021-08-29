use super::{css_ident::CssIdent, output::OutputFragment};
use proc_macro2::{Literal, Punct, Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use std::ops::Deref;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Error as ParseError, Parse, ParseBuffer, Result as ParseResult},
    token, Expr, ExprLit, Ident, Lit, LitStr,
};

#[derive(Debug, Clone)]
pub enum PreservedToken {
    Punct(Punct),
    Literal(Literal),
    Ident(CssIdent),
}

#[derive(Debug, Clone)]
pub enum SimpleBlock {
    Braced {
        brace: token::Brace,
        contents: Vec<ComponentValue>,
    },
    Bracketed {
        bracket: token::Bracket,
        contents: Vec<ComponentValue>,
    },
    Paren {
        paren: token::Paren,
        contents: Vec<ComponentValue>,
    },
}

// Already the consumed version of a function, not the parsed one.
// This should not be a problem since we make no effort to handle
// the insane special handling of 'url()' functions that's in the
// css syntax spec
#[derive(Debug, Clone)]
pub struct FunctionToken {
    name: CssIdent,
    paren: token::Paren,
    args: Vec<ComponentValue>,
}

#[derive(Debug, Clone)]
pub enum ComponentValue {
    Function(FunctionToken),
    Token(PreservedToken),
    Block(SimpleBlock),
    Expr(InjectedExpression),
}

#[derive(Debug, Clone)]
pub struct InjectedExpression {
    dollar: token::Dollar,
    braces: token::Brace,
    expr: Box<Expr>,
}

impl ToTokens for PreservedToken {
    fn to_tokens(&self, toks: &mut TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(toks),
            Self::Literal(i) => i.to_tokens(toks),
            Self::Punct(i) => i.to_tokens(toks),
        }
    }
}

impl ToTokens for SimpleBlock {
    fn to_tokens(&self, toks: &mut TokenStream) {
        match self {
            Self::Braced { brace, contents } => brace.surround(toks, |toks| {
                for c in contents.iter() {
                    c.to_tokens(toks);
                }
            }),
            Self::Bracketed { bracket, contents } => bracket.surround(toks, |toks| {
                for c in contents.iter() {
                    c.to_tokens(toks);
                }
            }),
            Self::Paren { paren, contents } => paren.surround(toks, |toks| {
                for c in contents.iter() {
                    c.to_tokens(toks);
                }
            }),
        }
    }
}

impl ToTokens for FunctionToken {
    fn to_tokens(&self, toks: &mut TokenStream) {
        self.name.to_tokens(toks);
        self.paren.surround(toks, |toks| {
            for c in self.args.iter() {
                c.to_tokens(toks);
            }
        });
    }
}

impl ToTokens for ComponentValue {
    fn to_tokens(&self, toks: &mut TokenStream) {
        match self {
            Self::Block(b) => b.to_tokens(toks),
            Self::Function(f) => f.to_tokens(toks),
            Self::Token(t) => t.to_tokens(toks),
            Self::Expr(e) => e.to_tokens(toks),
        }
    }
}

impl ToTokens for InjectedExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.dollar.to_tokens(tokens);
        self.braces.surround(tokens, |toks| {
            self.expr.to_tokens(toks);
        });
    }
}

impl Parse for PreservedToken {
    fn parse(input: &ParseBuffer) -> ParseResult<Self> {
        if CssIdent::peek(input) {
            Ok(Self::Ident(input.parse()?))
        } else if input.cursor().punct().is_some() {
            Ok(Self::Punct(input.parse()?))
        } else if input.cursor().literal().is_some() {
            Ok(Self::Literal(input.parse()?))
        } else {
            Err(input.error("Expected a css identifier, punctuation or a literal"))
        }
    }
}

impl Parse for SimpleBlock {
    fn parse(input: &ParseBuffer) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Brace) {
            let inside;
            let brace = braced!(inside in input);
            let contents = ComponentValue::parse_multiple(&inside)?;
            Ok(Self::Braced { brace, contents })
        } else if lookahead.peek(token::Bracket) {
            let inside;
            let bracket = bracketed!(inside in input);
            let contents = ComponentValue::parse_multiple(&inside)?;
            Ok(Self::Bracketed { bracket, contents })
        } else if lookahead.peek(token::Paren) {
            let inside;
            let paren = parenthesized!(inside in input);
            let contents = ComponentValue::parse_multiple(&inside)?;
            Ok(Self::Paren { paren, contents })
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for FunctionToken {
    fn parse(input: &ParseBuffer) -> ParseResult<Self> {
        Self::parse_with_name(input.parse()?, input)
    }
}

impl Parse for InjectedExpression {
    fn parse(input: &ParseBuffer) -> ParseResult<Self> {
        let dollar = input.parse()?;
        let inner;
        let braces = braced!(inner in input);
        let expr = Box::new(inner.parse()?);
        Ok(InjectedExpression {
            dollar,
            braces,
            expr,
        })
    }
}

impl FunctionToken {
    fn parse_with_name(name: CssIdent, input: &ParseBuffer) -> ParseResult<Self> {
        let inner;
        let paren = parenthesized!(inner in input);
        let args = ComponentValue::parse_multiple(&inner)?;
        Ok(Self { name, paren, args })
    }
}

impl Parse for ComponentValue {
    fn parse(input: &ParseBuffer) -> ParseResult<Self> {
        let is_group =
            input.peek(token::Brace) || input.peek(token::Bracket) || input.peek(token::Paren);
        if is_group {
            Ok(Self::Block(input.parse()?))
        } else if input.peek(token::Dollar) && input.peek2(token::Brace) {
            Ok(Self::Expr(input.parse()?))
        } else if !CssIdent::peek(input) {
            Ok(Self::Token(input.parse()?))
        } else {
            let ident = input.parse()?;
            if input.peek(token::Paren) {
                Ok(Self::Function(FunctionToken::parse_with_name(
                    ident, input,
                )?))
            } else {
                Ok(Self::Token(PreservedToken::Ident(ident)))
            }
        }
    }
}

#[derive(Debug)]
pub struct ComponentValueStream<'a> {
    input: &'a ParseBuffer<'a>,
}

impl<'a> From<&'a ParseBuffer<'a>> for ComponentValueStream<'a> {
    fn from(input: &'a ParseBuffer<'a>) -> Self {
        Self { input }
    }
}

impl<'a> From<ComponentValueStream<'a>> for &'a ParseBuffer<'a> {
    fn from(stream: ComponentValueStream<'a>) -> Self {
        stream.input
    }
}

impl<'a> Deref for ComponentValueStream<'a> {
    type Target = ParseBuffer<'a>;
    fn deref(&self) -> &Self::Target {
        self.input
    }
}

impl<'a> Iterator for ComponentValueStream<'a> {
    type Item = ParseResult<ComponentValue>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }
        Some(self.input.parse())
    }
}

impl ComponentValue {
    fn parse_multiple(input: &ParseBuffer) -> ParseResult<Vec<Self>> {
        ComponentValueStream::from(input).collect()
    }
}

impl InjectedExpression {
    pub fn to_output_fragment(&self) -> OutputFragment {
        let injected = &self.expr;
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(ref litstr),
            ..
        }) = **injected
        {
            return OutputFragment::Str(litstr.clone());
        }

        let ident_result = Ident::new("expr", Span::mixed_site());
        let ident_write_expr = Ident::new("write_expr", Span::mixed_site());
        let quoted = quote_spanned! {self.braces.span=>
            {
                fn #ident_write_expr<V: ::std::fmt::Display>(v: V) -> ::std::string::String {
                    use ::std::fmt::Write;
                    let mut #ident_result = ::std::string::String::new();
                    ::std::write!(&mut #ident_result, "{}", v).expect("");
                    #ident_result
                }
                #ident_write_expr(#injected).into()
            }
        };
        OutputFragment::Raw(quoted)
    }
}

impl PreservedToken {
    pub fn quote_literal(&self) -> LitStr {
        match self {
            Self::Ident(i) => i.quote_literal(),
            Self::Literal(l) => LitStr::new(&format!("{}", l), l.span()),
            Self::Punct(p) => LitStr::new(&format!("{}", p.as_char()), p.span()),
        }
    }
}

impl ComponentValue {
    // Reifies into a Vec of TokenStreams of type
    // for<I: Into<Cow<'static, str>>> T: From<I>
    // including ::stylist::ast::Selector and ::stylist::ast::StringFragment
    pub fn to_output_fragments(&self) -> impl '_ + IntoIterator<Item = OutputFragment> {
        use std::iter::once;
        match self {
            Self::Token(token) => {
                Box::new(once(token.clone().into())) as Box<dyn Iterator<Item = _>>
            }
            Self::Expr(expr) => Box::new(once(expr.to_output_fragment())),
            Self::Block(SimpleBlock::Bracketed { contents, .. }) => {
                let inner_parts = contents.iter().flat_map(|c| c.to_output_fragments());
                Box::new(once('['.into()).chain(inner_parts).chain(once(']'.into())))
            }
            Self::Block(SimpleBlock::Paren { contents, .. }) => {
                let inner_parts = contents.iter().flat_map(|c| c.to_output_fragments());
                Box::new(once('('.into()).chain(inner_parts).chain(once(')'.into())))
            }
            Self::Function(FunctionToken { name, args, .. }) => {
                let inner_args = args.iter().flat_map(|arg| arg.to_output_fragments());
                Box::new(
                    once(name.clone().into())
                        .chain(once('('.into()))
                        .chain(inner_args)
                        .chain(once(')'.into())),
                )
            }
            Self::Block(SimpleBlock::Braced { .. }) => {
                unreachable!("blocks should not get reified");
            }
        }
    }
    // Overly simplified parsing of a css attribute
    pub fn is_attribute_token(&self) -> bool {
        match self {
            Self::Expr(_)
            | Self::Token(PreservedToken::Ident(_))
            | Self::Token(PreservedToken::Literal(_)) => true,
            Self::Function(FunctionToken { args, .. }) => {
                args.iter().all(|a| a.is_attribute_token())
            }
            Self::Block(_) => false,
            Self::Token(PreservedToken::Punct(p)) => "-/%:,#".contains(p.as_char()),
        }
    }

    // Overly simplified of parsing a css selector :)
    pub fn validate_selector_token(&self) -> ParseResult<impl IntoIterator<Item = ParseError>> {
        match self {
            Self::Expr(_) | Self::Function(_) | Self::Token(PreservedToken::Ident(_)) => Ok(vec![]),
            Self::Block(SimpleBlock::Bracketed { contents, .. }) => {
                let mut collected = vec![];
                for e in contents.iter().map(|e| e.validate_selector_token()) {
                    collected.extend(e?);
                }
                Ok(collected)
            }
            Self::Block(_) => Ok(vec![ParseError::new_spanned(
                self,
                "expected a valid part of a scope qualifier, not a block",
            )]),
            Self::Token(PreservedToken::Literal(l)) => {
                let syn_lit = Lit::new(l.clone());
                if !matches!(syn_lit, Lit::Str(_)) {
                    Ok(vec![ParseError::new_spanned(
                        self,
                        "only string literals are allowed in selectors",
                    )])
                } else {
                    Ok(vec![])
                }
            }
            Self::Token(PreservedToken::Punct(p)) => {
                if p.as_char() == ';' {
                    Err(ParseError::new_spanned(
                        self,
                        "unexpected ';' in selector, did you mean to write an attribute?",
                    ))
                } else if !"&>+~|$*=^#.:,".contains(p.as_char()) {
                    Ok(vec![ParseError::new_spanned(
                        self,
                        "unexpected punctuation in selector",
                    )])
                } else {
                    Ok(vec![])
                }
            }
        }
    }
}

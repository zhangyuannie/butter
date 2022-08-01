use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{braced, parenthesized, parse::Parse, parse_macro_input, Token};
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let service = parse_macro_input!(item as Service);
    TokenStream::from(service.into_token_stream())
}

struct Service {
    vis: syn::Visibility,
    trait_token: Token![trait],
    ident: syn::Ident,
    brace_token: syn::token::Brace,
    items: Vec<Method>,
    request: Request,
}

impl Parse for Service {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let trait_token = input.parse()?;
        let ident = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }

        let request = Request::new(&ident, &items);

        Ok(Self {
            vis,
            trait_token,
            ident,
            brace_token,
            items,
            request,
        })
    }
}

impl ToTokens for Service {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.vis.to_tokens(tokens);
        self.trait_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            tokens.append_all(&self.items);

            let serve = service_serve(&self.request, &self.items);
            serve.to_tokens(tokens);
        });

        self.request.to_tokens(tokens);

        let client = service_client(self);
        client.to_tokens(tokens);
    }
}

#[derive(Clone)]
struct Method {
    fn_token: Token![fn],
    ident: syn::Ident,
    paren_token: syn::token::Paren,
    inputs: Vec<(syn::PatIdent, syn::Type)>,
    output: syn::ReturnType,
    semi_token: Token![;],
}

impl Parse for Method {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fn_token = input.parse()?;
        let ident = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let mut inputs = Vec::new();
        for arg in content.parse_terminated::<syn::FnArg, syn::token::Comma>(syn::FnArg::parse)? {
            if let syn::FnArg::Typed(arg) = arg {
                let typ = *arg.ty.clone();
                if let syn::Pat::Ident(ident) = *arg.pat {
                    inputs.push((ident, typ))
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        }
        let output = input.parse()?;
        let semi_token = input.parse()?;

        Ok(Method {
            fn_token,
            ident,
            paren_token,
            inputs,
            output,
            semi_token,
        })
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.fn_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            let iter = self.inputs.iter().map(|(ident, typ)| {
                quote! {
                    #ident: #typ
                }
            });
            let expanded = quote! {
                &mut self, #(#iter,)*
            };
            expanded.to_tokens(tokens);
        });
        self.output.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

struct Request {
    ident: syn::Ident,
    variants: Vec<RequestVariant>,
}

struct RequestVariant {
    ident: syn::Ident,
    fields: Vec<RequestField>,
    method: usize,
}

impl RequestVariant {
    fn typed_fields(&self) -> TokenStream2 {
        let fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                #ident: #ty
            }
        });
        quote! {
            #(#fields,)*
        }
    }

    fn fields(&self) -> TokenStream2 {
        let fields = self.fields.iter().map(|field| &field.ident);
        quote! {
            #(#fields,)*
        }
    }
}

struct RequestField {
    ident: syn::Ident,
    ty: syn::Type,
}

impl Request {
    fn new(service_ident: &syn::Ident, methods: &[Method]) -> Self {
        Request {
            ident: format_ident!("{}Request", service_ident),
            variants: methods
                .iter()
                .enumerate()
                .map(|(idx, method)| RequestVariant {
                    ident: format_ident!("{}", snake_to_pascal(method.ident.to_string())),
                    fields: method
                        .inputs
                        .iter()
                        .map(|(pat, ty)| RequestField {
                            ident: pat.ident.clone(),
                            ty: ty.clone(),
                        })
                        .collect(),
                    method: idx,
                })
                .collect(),
        }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let attrs = quote! {
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
        };
        attrs.to_tokens(tokens);
        <Token![pub]>::default().to_tokens(tokens);
        <Token![enum]>::default().to_tokens(tokens);
        self.ident.to_tokens(tokens);
        syn::token::Brace::default().surround(tokens, |tokens| {
            let variants = self.variants.iter().map(|variant| {
                let ident = &variant.ident;
                let fields = variant.typed_fields();
                quote! {
                    #ident { #fields }
                }
            });

            let expanded = quote! {
                #(#variants,)*
            };
            expanded.to_tokens(tokens);
        });
    }
}

fn service_serve(request: &Request, methods: &[Method]) -> TokenStream2 {
    let request_type = &request.ident;
    let arms = request.variants.iter().map(|variant| {
        let ident = &variant.ident;
        let method_name = &methods[variant.method].ident;
        let args = variant.fields();
        quote! {
            #request_type::#ident { #args } => serde_json::to_string(&self.#method_name(#args))
        }
    });
    quote! {
        fn serve(&mut self) {
            use std::io::BufRead;
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                let line = line.unwrap();
                if line.is_empty() {
                    break;
                }
                let req: #request_type = serde_json::from_str(&line).unwrap();
                let res = self.on_request(req);
                println!("{}", res);
            }
        }

        fn on_request(&mut self, req: #request_type) -> String {
            eprintln!("{:?}", req);
            let res = match req {
                #(#arms,)*
            };
            res.expect("failed to serialize response")
        }
    }
}

fn service_client(service: &Service) -> TokenStream2 {
    let ident = format_ident!("{}Client", service.ident);
    let request_type = &service.request.ident;

    let methods = service.request.variants.iter().map(|variant| {
        let method_name = &service.items[variant.method].ident;
        let typed_fields = variant.typed_fields();
        let return_type = &service.items[variant.method].output;
        let fields = variant.fields();
        let request_name = &variant.ident;

        quote! {
            pub fn #method_name(&mut self, #typed_fields) #return_type {
                serde_json::from_slice(&self.run(#request_type::#request_name { #fields })).unwrap()
            }
        }
    });

    quote! {
        #[derive(Debug)]
        pub struct #ident {
            pub reader: std::io::BufReader<std::process::ChildStdout>,
            pub writer: std::process::ChildStdin,
        }

        impl #ident {
            pub fn run(&mut self, request: #request_type) -> Vec<u8> {
                use std::io::{BufRead, Write};
                let req = serde_json::to_string(&request).unwrap();
                writeln!(self.writer, "{}", req).unwrap();
                let mut ret = Vec::new();
                let byte_count = self.reader.read_until('\n' as u8, &mut ret).unwrap();
                if byte_count == 0 {
                    println!("Daemon exited unexpectedly!");
                    std::process::exit(1);
                }
                ret
            }

            #(#methods)*
        }
    }
}

fn snake_to_pascal(s: String) -> String {
    let mut ret = String::with_capacity(s.len());
    let mut iter = s.chars();
    ret.push(iter.next().unwrap().to_ascii_uppercase());
    while let Some(char) = iter.next() {
        if char == '_' {
            ret.push(iter.next().unwrap().to_ascii_uppercase())
        } else {
            ret.push(char)
        }
    }
    ret
}

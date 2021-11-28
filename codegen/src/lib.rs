use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, Result};

mod tests;

#[proc_macro_derive(Gusket, attributes(gusket))]
pub fn gusket(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match gusket_impl(ts.into()) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn gusket_impl(ts: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<syn::DeriveInput>(ts)?;
    let input_ident = &input.ident;

    let mut input_attrs = InputAttrs::new(&input.vis);

    for attr in &input.attrs {
        if attr.path.is_ident("gusket") {
            input_attrs.apply(attr)?;
        }
    }

    let (generics_decl, generics_usage) = if input.generics.params.is_empty() {
        (quote!(), quote!())
    } else {
        let decl: Vec<_> = input.generics.params.iter().collect();
        let usage: Vec<_> = input
            .generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(syn::TypeParam { ident, .. }) => quote!(#ident),
                syn::GenericParam::Lifetime(syn::LifetimeDef { lifetime, .. }) => {
                    quote!(#lifetime)
                }
                syn::GenericParam::Const(syn::ConstParam { ident, .. }) => quote!(#ident),
            })
            .collect();
        (quote!(<#(#decl),*>), quote!(<#(#usage),*>))
    };
    let generics_where = &input.generics.where_clause;

    let data = match &input.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(data) => {
            return Err(Error::new_spanned(&data.enum_token, "Enums are not supported"));
        }
        syn::Data::Union(data) => {
            return Err(Error::new_spanned(&data.union_token, "Unions are not supported"));
        }
    };

    let named = match &data.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(fields) => {
            return Err(Error::new(fields.paren_token.span, "Tuple structs are not supported"));
        }
        syn::Fields::Unit => {
            return Err(Error::new_spanned(&data.semi_token, "Tuple structs are not supported"));
        }
    };

    let mut methods = TokenStream::new();

    for field in &named.named {
        process_field(field, &input_attrs, &mut methods)?;
    }

    let output = quote! {
        impl #generics_decl #input_ident #generics_usage #generics_where {
            #methods
        }
    };

    Ok(output)
}

fn process_field(
    field: &syn::Field,
    input_attrs: &InputAttrs,
    methods: &mut TokenStream,
) -> Result<()> {
    let field_ident = field.ident.as_ref().expect("Struct is named");
    let field_ty = &field.ty;

    let mut field_vis = input_attrs.vis.clone();
    let mut is_copy = None;
    let mut derive = input_attrs.derive;
    let mut mutable = input_attrs.mutable;

    let mut docs = Vec::new();

    for attr in &field.attrs {
        if attr.path.is_ident("gusket") {
            derive = true;

            if !attr.tokens.is_empty() {
                let attr_list: Punctuated<FieldAttr, syn::Token![,]> =
                    attr.parse_args_with(Punctuated::parse_terminated)?;
                for attr in attr_list {
                    match attr {
                        FieldAttr::Vis(_, vis) => field_vis = vis,
                        FieldAttr::Immut(_) => mutable = false,
                        FieldAttr::Mut(_) => mutable = true,
                        FieldAttr::Copy(ident) => is_copy = Some(ident),
                        FieldAttr::Skip(_) => derive = false,
                    }
                }
            }
        } else if attr.path.is_ident("doc") {
            docs.push(attr);
        }
    }

    if !derive {
        return Ok(());
    }

    let ref_op = match is_copy {
        Some(_) => quote!(),
        None => quote_spanned!(field.span() => &),
    };

    methods.extend(quote_spanned! { field.span() =>
        #(#docs)*
        #[must_use = "Getters have no side effect"]
        #[inline(always)]
        #field_vis fn #field_ident(&self) -> #ref_op #field_ty {
            #ref_op self.#field_ident
        }
    });

    if mutable {
        let setter = format_ident!("set_{}", &field_ident);
        let mut_getter = format_ident!("{}_mut", &field_ident);

        methods.extend(quote_spanned! { field.span() =>
            #(#docs)*
            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            #field_vis fn #mut_getter(&mut self) -> &mut #field_ty {
                &mut self.#field_ident
            }

            #(#docs)*
            #[inline(always)]
            #field_vis fn #setter(&mut self, #field_ident: #field_ty) {
                self.#field_ident = #field_ident;
            }
        })
    }

    Ok(())
}

struct InputAttrs {
    vis:     syn::Visibility,
    mutable: bool,
    derive:  bool,
}

impl InputAttrs {
    fn new(vis: &syn::Visibility) -> Self {
        InputAttrs { vis: vis.clone(), mutable: true, derive: false }
    }

    fn apply(&mut self, attr: &syn::Attribute) -> Result<()> {
        let attr_list: Punctuated<InputAttr, syn::Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for attr in attr_list {
            match attr {
                InputAttr::Vis(_, vis) => self.vis = vis,
                InputAttr::Immut(_) => self.mutable = false,
                InputAttr::All(_) => self.derive = true,
            }
        }

        Ok(())
    }
}

enum InputAttr {
    Vis(syn::Ident, syn::Visibility),
    Immut(syn::Ident),
    All(syn::Ident),
}

impl Parse for InputAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident == "vis" {
            input.parse::<syn::Token![=]>()?;
            let vis: syn::Visibility = input.parse()?;
            Ok(Self::Vis(ident, vis))
        } else if ident == "immut" {
            Ok(Self::Immut(ident))
        } else if ident == "all" {
            Ok(Self::All(ident))
        } else {
            Err(Error::new_spanned(ident, "Unsupported attribute"))
        }
    }
}

enum FieldAttr {
    Vis(syn::Ident, syn::Visibility),
    Immut(syn::Ident),
    Mut(syn::Token![mut]),
    Copy(syn::Ident),
    Skip(syn::Ident),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::Token![mut]) {
            let mut_token: syn::Token![mut] = input.parse()?;
            return Ok(Self::Mut(mut_token));
        }

        let ident: syn::Ident = input.parse()?;
        if ident == "vis" {
            input.parse::<syn::Token![=]>()?;
            let vis: syn::Visibility = input.parse()?;
            Ok(Self::Vis(ident, vis))
        } else if ident == "immut" {
            Ok(Self::Immut(ident))
        } else if ident == "copy" {
            Ok(Self::Copy(ident))
        } else if ident == "skip" {
            Ok(Self::Skip(ident))
        } else {
            Err(Error::new_spanned(ident, "Unsupported attribute"))
        }
    }
}

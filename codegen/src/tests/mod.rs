#![cfg(test)]

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream};

use crate::{process_field, InputAttrs};

fn token_stream_equals(ts1: TokenStream, ts2: TokenStream) -> bool {
    let mut ts1 = ts1.into_iter().fuse();
    let mut ts2 = ts2.into_iter().fuse();

    loop {
        match (ts1.next(), ts2.next()) {
            (Some(tt1), Some(tt2)) => match (tt1, tt2) {
                (TokenTree::Ident(i1), TokenTree::Ident(i2)) => {
                    if i1 != i2 {
                        return false;
                    }
                }
                (TokenTree::Punct(p1), TokenTree::Punct(p2)) => {
                    if p1.as_char() != p2.as_char() {
                        return false;
                    }
                }
                (TokenTree::Literal(l1), TokenTree::Literal(l2)) => {
                    if l1.to_string() != l2.to_string() {
                        return false;
                    }
                }
                (TokenTree::Group(g1), TokenTree::Group(g2)) => {
                    if !token_stream_equals(g1.stream(), g2.stream()) {
                        return false;
                    }
                }
                _ => return false,
            },
            (None, None) => return true,
            (Some(tt1), None) => {
                if let TokenTree::Punct(p1) = tt1 {
                    if p1.as_char() == ',' && ts1.next().is_none() {
                        return true;
                    }
                }
                return false;
            }
            (None, Some(tt2)) => {
                if let TokenTree::Punct(p2) = tt2 {
                    if p2.as_char() == ',' && ts2.next().is_none() {
                        return true;
                    }
                }
                return false;
            }
        }
    }
}

fn test_process_field(
    vis: TokenStream,
    input_attrs_ts: TokenStream,
    field: TokenStream,
    expect: TokenStream,
) {
    let mut input_attrs =
        InputAttrs::new(&syn::parse2(vis).expect("Invalid test input (visibility)"));

    struct AttrVecParse(Vec<syn::Attribute>);
    impl Parse for AttrVecParse {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let attrs = input.call(syn::Attribute::parse_outer)?;
            Ok(Self(attrs))
        }
    }

    for attr in syn::parse2::<AttrVecParse>(input_attrs_ts)
        .expect("Invalid test input (container attributes)")
        .0
    {
        input_attrs.apply(&attr).expect("Invalid test input (container attributes)");
    }

    struct NamedFieldParse(syn::Field);
    impl Parse for NamedFieldParse {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(Self(input.call(syn::Field::parse_named)?))
        }
    }

    let field = syn::parse2::<NamedFieldParse>(field).expect("Invalid test input (field)").0;

    let mut methods = TokenStream::new();
    process_field(&field, &input_attrs, &mut methods).expect("Error processing field");

    if !token_stream_equals(expect.clone(), methods.clone()) {
        panic!("Expected:\n{}\n\nGot:\n{}", expect, methods);
    }
}

#[test]
fn test_default() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {},
        quote! {
            #[gusket]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in some::module) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_container_immut() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(immut)]
        },
        quote! {
            #[gusket]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }
        },
    );
}

#[test]
fn test_field_immut() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {},
        quote! {
            #[gusket(immut)]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }
        },
    );
}

#[test]
fn test_container_immut_field_mut() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(immut)]
        },
        quote! {
            #[gusket(mut)]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in some::module) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_default_skip() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {},
        quote! {
            foo: Bar
        },
        quote! {},
    );
}

#[test]
fn test_immut_mut_override() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(immut)]
        },
        quote! {
            #[gusket(mut)]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in some::module) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_doc_copy() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {},
        quote! {
            /// This is some
            /// multiline documentation.
            #[gusket]
            foo: Bar
        },
        quote! {
            /// This is some
            /// multiline documentation.
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }

            /// This is some
            /// multiline documentation.
            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            /// This is some
            /// multiline documentation.
            #[inline(always)]
            pub(in some::module) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_container_all() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(all)]
        },
        quote! {
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> &Bar {
                &self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in some::module) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_container_all_skip() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(all)]
        },
        quote! {
            #[gusket(skip)]
            foo: Bar
        },
        quote! {},
    );
}

#[test]
fn test_container_vis_override() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(vis = pub(in something::different))]
        },
        quote! {
            #[gusket]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in something::different) fn foo(&self) -> &Bar {
                &self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in something::different) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in something::different) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_container_vis_override_field_vis_override() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {
            #[gusket(vis = pub(in something::different))]
        },
        quote! {
            #[gusket(vis = pub(in yet::another::path))]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in yet::another::path) fn foo(&self) -> &Bar {
                &self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in yet::another::path) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in yet::another::path) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

#[test]
fn test_copy() {
    test_process_field(
        quote!(pub(in some::module)),
        quote! {},
        quote! {
            #[gusket(copy)]
            foo: Bar
        },
        quote! {
            #[must_use = "Getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo(&self) -> Bar {
                self.foo
            }

            #[must_use = "Mutable getters have no side effect"]
            #[inline(always)]
            pub(in some::module) fn foo_mut(&mut self) -> &mut Bar {
                &mut self.foo
            }

            #[inline(always)]
            pub(in some::module) fn set_foo(&mut self, foo: Bar) {
                self.foo = foo;
            }
        },
    );
}

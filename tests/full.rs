#![allow(clippy::blacklisted_name)]
#![deny(dead_code, missing_docs)]

use gusket::Gusket;

#[derive(Default, Gusket)]
#[gusket(immut)]
struct Alpha {
    #[gusket]
    foo:    String,
    #[gusket(copy)]
    bar:    u32,
    #[gusket(copy, mut)]
    qux:    i32,
    #[gusket(mut)]
    corge:  Vec<u32>,
    grault: Option<u32>,
}

fn use_alpha(mut alpha: Alpha) {
    let _: &String = alpha.foo();

    let _: u32 = alpha.bar();

    let _: i32 = alpha.qux();
    let _: &mut i32 = alpha.qux_mut();
    alpha.set_qux(0i32);

    let _: &Vec<u32> = alpha.corge();
    let _: &mut Vec<u32> = alpha.corge_mut();

    let _ = &alpha.grault; // no getter method

    alpha.set_corge(vec![]);
}

#[derive(Default, Gusket)]
#[gusket(all)]
struct Beta {
    #[gusket(immut, vis = pub(self))]
    foo: String,
    #[gusket(skip)]
    bar: u32,
}

fn use_beta(beta: Beta) {
    let _: &String = beta.foo();

    let _ = &beta.bar; // no getter method
}

#[test]
fn test() {
    use_alpha(Alpha::default());
    use_beta(Beta::default());
}

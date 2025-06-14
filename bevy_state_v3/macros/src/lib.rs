//! Macros for deriving `State`.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate proc_macro;

//use bevy_macro_utils::BevyManifest;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Ident, ImplGenerics, Pat, Path, Result, TypeGenerics, WhereClause,
    parse_macro_input, spanned::Spanned,
};

pub(crate) fn bevy_state_path() -> Path {
    //BevyManifest::default().get_path("bevy_state_v3")
    syn::parse("bevy_state_v3".parse::<TokenStream>().unwrap()).unwrap()
}

struct Dependency {
    ty: Path,
    value: Pat,
}

fn parse_sources_attr(ast: &DeriveInput) -> Result<Option<Dependency>> {
    let mut result = ast
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("dependency"))
        .map(|meta| {
            let mut source = None;
            let value = meta.parse_nested_meta(|nested| {
                let ty = nested.path.clone();
                let value = Pat::parse_multi(nested.value()?)?;
                source = Some(Dependency { ty, value });
                Ok(())
            });
            match source {
                Some(value) => Ok(value),
                None => match value {
                    Ok(_) => Err(syn::Error::new(ast.span(), "couldn't parse dependency")),
                    Err(e) => Err(e),
                },
            }
        })
        .collect::<Result<Vec<_>>>()?;

    if result.len() > 1 {
        return Err(syn::Error::new(
            ast.span(),
            "only one state is allowed as dependency",
        ));
    }

    Ok(result.pop())
}

struct Shared<'a> {
    impl_generics: ImplGenerics<'a>,
    ty_generics: TypeGenerics<'a>,
    where_clause: Option<&'a WhereClause>,
    trait_path: Path,
    struct_name: &'a Ident,
}

/// Macro for deriving `State` trait.
///
/// By default, this will create a root state which:
/// - has no dependencies,
/// - is non-optional (always exists),
/// - mutation is done only through replacement with the new value.
///
/// If attributed with `#[dependency(MyState = MyState::Foo)]`, the state will:
/// - have one dependency (`MyState`),
/// - be optional (exists only if `MyState::Foo`),
/// - use default value for initial state,
/// - be mutated by replacement if exists.
#[proc_macro_derive(State, attributes(dependency))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let dependency = parse_sources_attr(&ast).expect("failed to parse dependency");

    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut base_path = bevy_state_path();
    base_path.segments.push(format_ident!("state").into());

    let mut trait_path = base_path.clone();
    trait_path.segments.push(format_ident!("State").into());

    let struct_name = &ast.ident;

    let shared = Shared {
        impl_generics,
        ty_generics,
        where_clause,
        trait_path,
        struct_name,
    };

    let result = match dependency {
        Some(dependency) => derive_sub_state(shared, dependency),
        None => derive_root_state(shared),
    };

    result.into()
}

fn derive_root_state(shared: Shared) -> proc_macro2::TokenStream {
    let Shared {
        impl_generics,
        ty_generics,
        where_clause,
        trait_path,
        struct_name,
    } = shared;
    quote! {
        impl #impl_generics #trait_path for #struct_name #ty_generics #where_clause {
            type Dependencies = ();
            type Update = Option<Self>;
            type Repr = Self;

            fn update<'a>(
                state: &mut StateData<Self>,
                _: StateSetData<'_, Self::Dependencies>,
            ) -> Self::Repr {
                state.update_mut().take().unwrap()
            }
        }
    }
}

fn derive_sub_state(shared: Shared, dependency: Dependency) -> proc_macro2::TokenStream {
    let Shared {
        impl_generics,
        ty_generics,
        where_clause,
        trait_path,
        struct_name,
    } = shared;
    let Dependency {
        ty: dependency_ty,
        value: dependency_value,
    } = dependency;
    quote! {
        impl #impl_generics #trait_path for #struct_name #ty_generics #where_clause {
            type Dependencies = #dependency_ty;
            type Update = Option<Self>;
            type Repr = Option<Self>;

            fn update<'a>(
                state: &mut StateData<Self>,
                dependencies: StateSetData<'_, Self::Dependencies>,
            ) -> Self::Repr {
                let manual = dependencies;
                match (manual.current(), state.update_mut().take()) {
                    (#dependency_value, None) => Some(Self::default()),
                    (#dependency_value, Some(next)) => Some(next),
                    _ => None,
                }
            }
        }
    }
}

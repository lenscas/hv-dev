use std::borrow::Cow;

use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Result};

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let data = match input.data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "derive(Bundle) does not support enums or unions",
            ))
        }
    };
    let (tys, field_members) = struct_fields(&data.fields);
    let field_idents = member_as_idents(&field_members);
    let generics = add_additional_bounds_to_generic_params(input.generics);

    let dyn_bundle_code = gen_dynamic_bundle_impl(&ident, &generics, &field_members, &tys);
    let bundle_code = if tys.is_empty() {
        gen_unit_struct_bundle_impl(ident, &generics)
    } else {
        gen_bundle_impl(&ident, &generics, &field_members, &field_idents, &tys)
    };
    let mut ts = dyn_bundle_code;
    ts.extend(bundle_code);
    Ok(ts)
}

fn gen_dynamic_bundle_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    field_members: &[syn::Member],
    tys: &[&syn::Type],
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        unsafe impl #impl_generics ::hv::ecs::DynamicBundle for #ident #ty_generics #where_clause {
            fn key(&self) -> ::core::option::Option<::core::any::TypeId> {
                ::core::option::Option::Some(::core::any::TypeId::of::<Self>())
            }

            fn with_ids<__hv::ecs__T>(&self, f: impl ::std::ops::FnOnce(&[::std::any::TypeId]) -> __hv::ecs__T) -> __hv::ecs__T {
                <Self as ::hv::ecs::Bundle>::with_static_ids(f)
            }

            fn type_info(&self) -> ::std::vec::Vec<::hv::ecs::TypeInfo> {
                <Self as ::hv::ecs::Bundle>::static_type_info()
            }

            #[allow(clippy::forget_copy)]
            unsafe fn put(mut self, mut f: impl ::std::ops::FnMut(*mut u8, ::hv::ecs::TypeInfo)) {
                #(
                    f((&mut self.#field_members as *mut #tys).cast::<u8>(), ::hv::ecs::TypeInfo::of::<#tys>());
                    ::std::mem::forget(self.#field_members);
                )*
            }
        }
    }
}

fn gen_bundle_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    field_members: &[syn::Member],
    field_idents: &[Cow<syn::Ident>],
    tys: &[&syn::Type],
) -> TokenStream2 {
    let num_tys = tys.len();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let with_static_ids_inner = quote! {
        {
            let mut tys = [#((::std::mem::align_of::<#tys>(), ::std::any::TypeId::of::<#tys>())),*];
            tys.sort_unstable_by(|x, y| {
                ::std::cmp::Ord::cmp(&x.0, &y.0)
                    .reverse()
                    .then(::std::cmp::Ord::cmp(&x.1, &y.1))
            });
            let mut ids = [::std::any::TypeId::of::<()>(); #num_tys];
            for (id, info) in ::std::iter::Iterator::zip(ids.iter_mut(), tys.iter()) {
                *id = info.1;
            }
            ids
        }
    };
    let with_static_ids_body = if generics.params.is_empty() {
        quote! {
            ::hv::ecs::lazy_static::lazy_static! {
                static ref ELEMENTS: [::std::any::TypeId; #num_tys] = {
                    #with_static_ids_inner
                };
            }
            f(&*ELEMENTS)
        }
    } else {
        quote! {
            f(&#with_static_ids_inner)
        }
    };
    quote! {
        unsafe impl #impl_generics ::hv::ecs::Bundle for #ident #ty_generics #where_clause {
            #[allow(non_camel_case_types)]
            fn with_static_ids<__hv::ecs__T>(f: impl ::std::ops::FnOnce(&[::std::any::TypeId]) -> __hv::ecs__T) -> __hv::ecs__T {
                #with_static_ids_body
            }

            fn static_type_info() -> ::std::vec::Vec<::hv::ecs::TypeInfo> {
                let mut info = ::std::vec![#(::hv::ecs::TypeInfo::of::<#tys>()),*];
                info.sort_unstable();
                info
            }

            unsafe fn get(
                mut f: impl ::std::ops::FnMut(::hv::ecs::TypeInfo) -> ::std::option::Option<::std::ptr::NonNull<u8>>,
            ) -> ::std::result::Result<Self, ::hv::ecs::MissingComponent> {
                #(
                    let #field_idents = f(::hv::ecs::TypeInfo::of::<#tys>())
                            .ok_or_else(::hv::ecs::MissingComponent::new::<#tys>)?
                            .cast::<#tys>()
                            .as_ptr();
                )*
                ::std::result::Result::Ok(Self { #( #field_members: #field_idents.read(), )* })
            }
        }
    }
}

// no reason to generate a static for unit structs
fn gen_unit_struct_bundle_impl(ident: syn::Ident, generics: &syn::Generics) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        unsafe impl #impl_generics ::hv::ecs::Bundle for #ident #ty_generics #where_clause {
            #[allow(non_camel_case_types)]
            fn with_static_ids<__hv::ecs__T>(f: impl ::std::ops::FnOnce(&[::std::any::TypeId]) -> __hv::ecs__T) -> __hv::ecs__T { f(&[]) }
            fn static_type_info() -> ::std::vec::Vec<::hv::ecs::TypeInfo> { ::std::vec::Vec::new() }

            unsafe fn get(
                mut f: impl ::std::ops::FnMut(::hv::ecs::TypeInfo) -> ::std::option::Option<::std::ptr::NonNull<u8>>,
            ) -> ::std::result::Result<Self, ::hv::ecs::MissingComponent> {
                ::std::result::Result::Ok(Self {/* for some reason this works for all unit struct variations */})
            }
        }
    }
}

fn make_component_trait_bound() -> syn::TraitBound {
    syn::TraitBound {
        paren_token: None,
        modifier: syn::TraitBoundModifier::None,
        lifetimes: None,
        path: syn::parse_quote!(::hv::ecs::Component),
    }
}

fn add_additional_bounds_to_generic_params(mut generics: syn::Generics) -> syn::Generics {
    generics.type_params_mut().for_each(|tp| {
        tp.bounds
            .push(syn::TypeParamBound::Trait(make_component_trait_bound()))
    });
    generics
}

fn struct_fields(fields: &syn::Fields) -> (Vec<&syn::Type>, Vec<syn::Member>) {
    match fields {
        syn::Fields::Named(ref fields) => fields
            .named
            .iter()
            .map(|f| (&f.ty, syn::Member::Named(f.ident.clone().unwrap())))
            .unzip(),
        syn::Fields::Unnamed(ref fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                (
                    &f.ty,
                    syn::Member::Unnamed(syn::Index {
                        index: i as u32,
                        span: Span::call_site(),
                    }),
                )
            })
            .unzip(),
        syn::Fields::Unit => (Vec::new(), Vec::new()),
    }
}

fn member_as_idents(members: &[syn::Member]) -> Vec<Cow<'_, syn::Ident>> {
    members
        .iter()
        .map(|member| match member {
            syn::Member::Named(ident) => Cow::Borrowed(ident),
            &syn::Member::Unnamed(syn::Index { index, span }) => {
                Cow::Owned(syn::Ident::new(&format!("tuple_field_{}", index), span))
            }
        })
        .collect()
}

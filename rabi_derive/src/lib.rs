use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Type, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(IntoRaw)]
pub fn derive_into_raw(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let raw_name = syn::Ident::new(&format!("Raw{}", name), name.span());

    let mut fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(named) => named.named.iter().cloned().collect::<Vec<Field>>(),
            Fields::Unnamed(_) | Fields::Unit => vec![],
        },
        Data::Enum(_) | Data::Union(_) => {
            return syn::Error::new_spanned(&input.ident, "incompatible data type")
                .to_compile_error()
                .into();
        }
    };
    let wrapped_fields = fields.iter().map(|f| {
        let vis = &f.vis;
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let wrapped_ty: Type = syn::parse2(quote! { rabi::Raw<#ty> }).unwrap();
        quote! {
            #vis #ident: #wrapped_ty
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();
    let into_raw_fields = fields.iter_mut().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &mut f.ty;
        let span = ty.span();
        match ty {
            syn::Type::Path(syn::TypePath { qself: None, path }) => {
                if let Some(last) = path.segments.last_mut()
                    && let syn::PathArguments::AngleBracketed(ab) = &mut last.arguments
                    && ab.colon2_token.is_none()
                {
                    ab.colon2_token = Some(syn::Token![::](span));
                }
                quote! {
                    #ident: #ty::into_raw(self.#ident)
                }
            }
            _ => quote! {
                #ident: #ty::into_raw(self.#ident)
            },
        }
    });
    TokenStream::from(quote! {
        #[repr(C)]
        pub struct #raw_name {
            #(#wrapped_fields,)*
        }

        impl rabi::RawRepr for #name {
            type Repr = #raw_name;
        }

        impl rabi::RawRepr for #raw_name {
            type Repr = #raw_name;
        }

        impl rabi::IntoRaw for #name {
            type Output = #raw_name;

            fn into_raw(self) -> rabi::InnerRaw<Self::Output> {
                rabi::InnerRaw {
                    value: std::mem::ManuallyDrop::new(
                        #raw_name {
                            #(#into_raw_fields,)*
                        }
                    )
                }
            }
        }
    })
}

#[proc_macro_derive(FromRaw)]
pub fn derive_from_raw(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let raw_name = syn::Ident::new(&format!("Raw{}", name), name.span());
    let mut fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(named) => named.named.iter().cloned().collect::<Vec<Field>>(),
            Fields::Unnamed(_) | Fields::Unit => vec![],
        },
        Data::Enum(_) | Data::Union(_) => {
            return syn::Error::new_spanned(&input.ident, "incompatible data type")
                .to_compile_error()
                .into();
        }
    };
    let from_raw_fields = fields.iter_mut().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &mut f.ty;
        let span = ty.span();
        match ty {
            syn::Type::Path(syn::TypePath { qself: None, path }) => {
                if let Some(last) = path.segments.last_mut()
                    && let syn::PathArguments::AngleBracketed(ab) = &mut last.arguments
                    && ab.colon2_token.is_none()
                {
                    ab.colon2_token = Some(syn::Token![::](span));
                }
                quote! {
                    #ident: #ty::from_raw(raw.#ident)
                }
            }
            _ => quote! {
                #ident: #ty::from_raw(raw.#ident)
            },
        }
    });
    TokenStream::from(quote! {
        impl rabi::FromRaw for #name {
            type Input = #raw_name;
            type Output = Self;

            fn from_raw(raw: rabi::InnerRaw<Self::Input>) -> Self::Output {
                let raw: #raw_name = std::mem::ManuallyDrop::into_inner(unsafe {
                    match raw {
                        rabi::InnerRaw { value } => value,
                    }
                });
                Self {
                    #(#from_raw_fields,)*
                }
            }
        }
    })
}

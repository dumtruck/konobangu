extern crate proc_macro;

use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField, ast::Data, util::Ignored};
use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Attribute, DeriveInput, Generics, Ident, parse_macro_input};

use crate::derives::attributes::related_attr;

#[derive(snafu::Snafu, Debug)]
enum GeneratorError {
    #[snafu(transparent)]
    Syn { source: syn::Error },

    #[snafu(transparent)]
    Darling { source: darling::Error },
}

impl GeneratorError {
    fn write_errors(self) -> proc_macro2::TokenStream {
        match self {
            GeneratorError::Syn { source } => source.to_compile_error(),
            GeneratorError::Darling { source } => source.write_errors(),
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(dyngql))]
#[allow(dead_code)]
struct DynamicGraphqlFieldInfo {
    ident: Option<Ident>,
    ty: syn::Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(dyngql), forward_attrs(doc))]
#[allow(dead_code)]
struct DynamicGraphqlInfo {
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
    pub data: Data<Ignored, DynamicGraphqlFieldInfo>,
}

impl DynamicGraphqlInfo {
    fn expand(&self) -> Result<TokenStream, GeneratorError> {
        let struct_name = &self.ident;
        let enum_name = format_ident!("{}FieldEnum", struct_name);

        let fields = self.data.as_ref().take_struct().unwrap();

        let enum_variants = fields
            .iter()
            .filter_map(|field| field.ident.as_ref())
            .map(|field_ident| {
                let variant_name = Ident::new(
                    &field_ident.to_string().to_case(Case::Pascal),
                    field_ident.span(),
                );
                quote! { #variant_name }
            })
            .collect::<Vec<_>>();

        let as_str_arms: Vec<_> = fields
            .iter()
            .filter_map(|field| field.ident.as_ref())
            .map(|field_ident| {
                let variant_name = Ident::new(
                    &field_ident.to_string().to_case(Case::Pascal),
                    field_ident.span(),
                );
                let field_name_str = field_ident.to_string().to_case(Case::Camel);
                quote! {
                    Self::#variant_name => #field_name_str,
                }
            })
            .collect::<Vec<_>>();

        let from_str_arms: Vec<_> = fields
            .iter()
            .filter_map(|field| field.ident.as_ref())
            .map(|field_ident| {
                let variant_name = Ident::new(
                    &field_ident.to_string().to_case(Case::Pascal),
                    field_ident.span(),
                );
                let field_name_str = field_ident.to_string().to_case(Case::Camel);
                quote! {
                    #field_name_str => Some(Self::#variant_name)
                }
            })
            .collect();

        let all_field_names: Vec<_> = fields
            .iter()
            .filter_map(|field| field.ident.as_ref())
            .map(|field_ident| field_ident.to_string().to_case(Case::Camel))
            .collect();

        let result = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_name {
                #(#enum_variants),*
            }

            impl #enum_name {
                pub fn as_str(&self) -> &'static str {
                    match self {
                        #(#as_str_arms),*
                    }
                }

                pub fn from_str(s: &str) -> Option<Self> {
                    match s {
                        #(#from_str_arms),* ,
                        _ => None
                    }
                }

                pub fn all_field_names() -> Vec<&'static str> {
                    vec![#(#all_field_names),*]
                }
            }

            impl std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.as_str())
                }
            }

            impl From<#enum_name> for String {
                fn from(value: #enum_name) -> Self {
                    value.as_str().to_string()
                }
            }

            impl std::str::FromStr for #enum_name {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Self::from_str(s).ok_or_else(|| format!("Unknown field name: {s}"))
                }
            }

        };

        Ok(result.into())
    }
}

#[proc_macro_derive(DynamicGraphql, attributes(dyngql))]
pub fn derive_dynamic_graphql(input: TokenStream) -> TokenStream {
    let opts =
        match DynamicGraphqlInfo::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
            Ok(opts) => opts,
            Err(err) => return TokenStream::from(err.write_errors()),
        };
    match opts.expand() {
        Ok(token_stream) => token_stream,
        Err(err) => err.write_errors().into(),
    }
}

enum Error {
    InputNotEnum,
    InvalidEntityPath,
    Syn(syn::Error),
}

struct DeriveRelatedEntity {
    entity_ident: TokenStream,
    ident: syn::Ident,
    variants: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
}

impl DeriveRelatedEntity {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let sea_attr = related_attr::SeaOrm::try_from_attributes(&input.attrs)
            .map_err(Error::Syn)?
            .unwrap_or_default();

        let ident = input.ident;
        let entity_ident = match sea_attr.entity.as_ref().map(Self::parse_lit_string) {
            Some(entity_ident) => entity_ident.map_err(|_| Error::InvalidEntityPath)?,
            None => quote! { Entity },
        };

        let variants = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        Ok(DeriveRelatedEntity {
            entity_ident,
            ident,
            variants,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        let entity_ident = &self.entity_ident;

        let variant_implementations: Vec<TokenStream> = self
                .variants
                .iter()
                .map(|variant| {
                    let attr = related_attr::SeaOrm::from_attributes(&variant.attrs)?;

                    let enum_name = &variant.ident;

                    let target_entity = attr
                        .entity
                        .as_ref()
                        .map(Self::parse_lit_string)
                        .ok_or_else(|| {
                            syn::Error::new_spanned(variant, "Missing value for 'entity'")
                        })??;

                    let def = match attr.def {
                        Some(def) => Some(Self::parse_lit_string(&def).map_err(|_| {
                            syn::Error::new_spanned(variant, "Missing value for 'def'")
                        })?),
                        None => None,
                    };

                    let name = enum_name.to_string().to_lower_camel_case();

                    if let Some(def) = def {
                        Result::<_, syn::Error>::Ok(quote! {
                            Self::#enum_name => builder.get_relation::<#entity_ident, #target_entity>(#name, #def)
                        })
                    } else {
                        Result::<_, syn::Error>::Ok(quote! {
                            Self::#enum_name => via_builder.get_relation::<#entity_ident, #target_entity>(#name)
                        })
                    }

                })
                .collect::<Result<Vec<_>, _>>()?;

        // Get the path of the `async-graphql` on the application's Cargo.toml
        let async_graphql_crate = match crate_name("async-graphql") {
            // if found, use application's `async-graphql`
            Ok(FoundCrate::Name(name)) => {
                let ident = Ident::new(&name, Span::call_site());
                quote! { #ident }
            }
            Ok(FoundCrate::Itself) => quote! { async_graphql },
            // if not, then use the `async-graphql` re-exported by `seaography`
            Err(_) => quote! { seaography::async_graphql },
        };

        Ok(quote! {
            impl seaography::RelationBuilder for #ident {
                fn get_relation(&self, context: & 'static seaography::BuilderContext) -> #async_graphql_crate::dynamic::Field {
                    let builder = seaography::EntityObjectRelationBuilder { context };
                    let via_builder = seaography::EntityObjectViaRelationBuilder { context };
                    match self {
                        #(#variant_implementations,)*
                        _ => panic!("No relations for this entity"),
                    }
                }

            }
        })
    }

    fn parse_lit_string(lit: &syn::Lit) -> syn::Result<TokenStream> {
        match lit {
            syn::Lit::Str(lit_str) => lit_str
                .value()
                .parse()
                .map_err(|_| syn::Error::new_spanned(lit, "attribute not valid")),
            _ => Err(syn::Error::new_spanned(lit, "attribute must be a string")),
        }
    }
}

/// Method to derive a Related enumeration
fn expand_derive_related_entity(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveRelatedEntity::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveRelation on enums");
        }),
        Err(Error::InvalidEntityPath) => Ok(quote_spanned! {
            ident_span => compile_error!("invalid attribute value for 'entity'");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}

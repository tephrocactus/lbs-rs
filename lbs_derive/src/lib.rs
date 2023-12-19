use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use quote::{quote, quote_spanned};
use std::{collections::HashSet, convert::TryInto};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DataEnum, DeriveInput,
    Fields, FieldsNamed, GenericParam, Generics,
};

const ATTR_LBS: &'static str = "lbs";
const ATTR_LBS_DEFAULT: &'static str = "lbs_default";
const FLAG_OMIT: &'static str = "omit";

//
// Derive LBSWrite
//

#[proc_macro_derive(LBSWrite, attributes(lbs))]
pub fn derive_lbs_write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Add trait bound to every generic type parameter
    let generics = add_write_trait_bound(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate lbs_write() body
    let write_body = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => generate_write_body_for_struct(fields),
            Fields::Unnamed(_) => unimplemented!(),
            Fields::Unit => quote!(Ok(())),
        },
        Data::Enum(ref data) => generate_write_body_for_enum(data),
        Data::Union(_) => unimplemented!(),
    };

    // Complete trait implementation
    proc_macro::TokenStream::from(quote! {
        impl #impl_generics lbs::LBSWrite for #name #ty_generics #where_clause {
            #[inline]
            fn lbs_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
                #write_body
            }

            #[inline]
            fn lbs_is_default(&self) -> bool {
                false
            }
        }
    })
}

//
// Derive LBSRead
//

#[proc_macro_derive(LBSRead, attributes(lbs, lbs_default))]
pub fn derive_lbs_read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Add trait bound LBSRead to every generic type parameter
    let generics = add_read_trait_bound(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate lbs_read() body
    let read_body = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => generate_read_body_for_struct(fields),
            Fields::Unnamed(_) => unimplemented!(),
            Fields::Unit => quote!(Ok(Self)),
        },
        Data::Enum(ref data) => generate_read_body_for_enum(data),
        Data::Union(_) => unimplemented!(),
    };

    // Complete trait implementation
    proc_macro::TokenStream::from(quote! {
        impl #impl_generics lbs::LBSRead for #name #ty_generics #where_clause {
            #[inline]
            fn lbs_read<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
                #read_body
            }
        }
    })
}

fn generate_write_body_for_struct(fields: &FieldsNamed) -> TokenStream {
    // Gather meta
    let meta = gather_struct_meta(fields);

    // Field count expressions
    let field_count_expressions = meta.iter().filter(|m| !m.omit).map(|m| {
        let field_name = &m.name;
        quote_spanned! {m.span=>
            if !self.#field_name.lbs_is_default() {
                field_count += 1;
            }
        }
    });

    // Write expressions
    let write_expressions = meta.iter().filter(|m| !m.omit).map(|m| {
        let field_id = m.id;
        let field_name = &m.name;
        quote_spanned! {m.span=>
            if !self.#field_name.lbs_is_default() {
                lbs::write::write_field_id(w, #field_id)?;
                self.#field_name.lbs_write(w)?;
            }
        }
    });

    // Complete body of lbs_write()
    quote! {
        let mut field_count: u16 = 0;

        #(#field_count_expressions)*

        lbs::write::write_field_count(w, field_count)?;

        if field_count > 0 {
            #(#write_expressions)*
        }

        Ok(())
    }
}

fn generate_write_body_for_enum(data: &DataEnum) -> TokenStream {
    // Gather meta
    let meta = gather_enum_meta(data);

    // Write expressions
    let write_expressions = meta.iter().map(|m| {
        let variant_id = m.id;
        let variant_name = &m.name;

        if let Some(_) = m.variant_fields {
            return quote_spanned! {m.span=>
                Self::#variant_name(inner) => {
                    lbs::write::write_field_id(w, #variant_id)?;
                    inner.lbs_write(w)?;
                },
            };
        }

        quote_spanned! {m.span=>
            Self::#variant_name => lbs::write::write_field_id(w, #variant_id)?,
        }
    });

    // Complete body of lbs_write()
    quote! {
        match self {
            #(#write_expressions)*
        }
        Ok(())
    }
}

fn generate_read_body_for_struct(fields: &FieldsNamed) -> TokenStream {
    // Gather meta
    let meta = gather_struct_meta(fields);

    // Field initialization expressions
    let field_init_expressions = meta.iter().map(|f| {
        let field_name = &f.name;
        match f.default {
            Some(ref default) => quote_spanned! {f.span=>
                #field_name: #default,
            },
            None => quote_spanned! {f.span=>
                #field_name: Default::default(),
            },
        }
    });

    // Read expressions
    let read_expressions = meta.iter().filter(|f| !f.omit).map(|f| {
        let field_id = f.id;
        let field_name = &f.name;
        quote_spanned! {f.span=>
            #field_id => _self.#field_name = lbs::read::read(r)?,
        }
    });

    // Complete body of lbs_read()
    quote! {
        let mut _self = Self {
            #(#field_init_expressions)*
        };

        for _ in 0..lbs::read::read_field_count(r)? {
            match lbs::read::read_field_id(r)? {
                #(#read_expressions)*
                _ => {},
            }
        }

        Ok(_self)
    }
}

fn generate_read_body_for_enum(data: &DataEnum) -> TokenStream {
    // Gather meta
    let meta = gather_enum_meta(data);

    // Read expressions
    let read_expressions = meta.iter().map(|m| {
        let variant_id = m.id;
        let variant_name = &m.name;

        if let Some(_) = m.variant_fields {
            return quote_spanned! {m.span=>
                #variant_id => Ok(Self::#variant_name(lbs::read::read(r)?)),
            };
        }

        quote_spanned! {m.span=>
            #variant_id => Ok(Self::#variant_name),
        }
    });

    // Complete body of lbs_read()
    quote! {
        match lbs::read::read_field_id(r)? {
            #(#read_expressions)*
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unknown enum variant"))
        }
    }
}

struct Meta {
    id: u16,
    name: Option<syn::Ident>,
    span: Span,
    omit: bool,
    default: Option<TokenStream>,
    variant_fields: Option<Fields>,
}

fn gather_struct_meta(fields: &FieldsNamed) -> Vec<Meta> {
    let mut metas = Vec::new();
    let mut unique_ids = HashSet::new();

    for (i, f) in fields.named.iter().enumerate() {
        metas.push(Meta {
            id: get_id(i, &f.attrs, &mut unique_ids),
            name: f.ident.clone(),
            span: f.span(),
            omit: has_omit_attr(&f.attrs),
            default: get_attr_code(&f.attrs, ATTR_LBS_DEFAULT),
            variant_fields: None,
        });
    }

    metas
}

fn gather_enum_meta(data: &DataEnum) -> Vec<Meta> {
    let mut metas = Vec::new();
    let mut unique_ids = HashSet::new();

    let panic_msg = "currently, only enums with single unnamed field are supported";

    for (i, v) in data.variants.iter().enumerate() {
        if v.fields.len() > 1 {
            panic!("{}", panic_msg);
        }

        match v.fields {
            Fields::Unit => {}
            Fields::Unnamed(_) => {}
            _ => panic!("{}", panic_msg),
        }

        metas.push(Meta {
            id: get_id(i, &v.attrs, &mut unique_ids),
            name: Some(v.ident.clone()),
            span: v.span(),
            omit: has_omit_attr(&v.attrs),
            default: get_attr_code(&v.attrs, ATTR_LBS_DEFAULT),
            variant_fields: if v.fields.is_empty() {
                None
            } else {
                Some(v.fields.clone())
            },
        });
    }

    metas
}

fn add_write_trait_bound(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(lbs::LBSWrite));
        }
    }
    generics
}

fn add_read_trait_bound(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(lbs::LBSRead));
        }
    }
    generics
}

fn get_id(index: usize, attrs: &[Attribute], unique_ids: &mut HashSet<u16>) -> u16 {
    let mut id: Option<u16> = None;

    // Try to find explicit ID defined via lbs attribute
    for attr in attrs {
        if attr.path().is_ident(ATTR_LBS) {
            if let Ok(ident) = attr.parse_args::<syn::LitInt>() {
                id = Some(ident.base10_parse().unwrap());
                break;
            }
        }
    }

    // If explicit ID wasn't found - use member index
    let id = if id.is_none() {
        index.try_into().unwrap()
    } else {
        id.unwrap()
    };

    // Ensure ID is unique
    if !unique_ids.insert(id) {
        panic!("duplicate id: {}", id);
    }

    id
}

fn get_attr_code(attrs: &[Attribute], attr_name: &str) -> Option<TokenStream> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            if let Ok(expr) = attr.parse_args::<syn::Expr>() {
                return Some(expr.to_token_stream());
            }
        }
    }
    None
}

fn has_omit_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident(ATTR_LBS) {
            if let Ok(ident) = attr.parse_args::<syn::Ident>() {
                return ident.eq(FLAG_OMIT);
            }
        }
    }
    false
}

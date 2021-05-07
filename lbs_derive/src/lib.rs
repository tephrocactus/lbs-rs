use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use std::{collections::HashSet, convert::TryInto};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Fields,
    GenericParam, Generics,
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

    // Generate method bodies
    let write_body = generate_write_body(&input.data);

    // Generate and return complete trait implementation
    proc_macro::TokenStream::from(quote! {
        impl #impl_generics lbs::LBSWrite for #name #ty_generics #where_clause {
            #[inline]
            fn lbs_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
                #write_body
            }

            #[inline]
            fn lbs_omit(&self) -> bool {
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

    // Generate method bodies
    let read_body = generate_read_body(&input.data);

    // Generate and return complete trait implementation
    proc_macro::TokenStream::from(quote! {
        impl #impl_generics lbs::LBSRead for #name #ty_generics #where_clause {
            #[inline]
            fn lbs_read<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
                #read_body
            }
        }
    })
}

fn generate_write_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                // Ensure there is no fields with same ID
                let mut unique_ids = HashSet::new();
                for (i, f) in fields
                    .named
                    .iter()
                    .filter(|f| !has_omit_attr(&f.attrs))
                    .enumerate()
                {
                    let id = get_id_attr(&f.attrs).unwrap_or(i.try_into().unwrap());
                    if !unique_ids.insert(id) {
                        panic!("duplicate id: {}", id);
                    }
                }

                // Preparation expressions
                let field_count_incr_expressions = fields
                    .named
                    .iter()
                    .filter(|f| !has_omit_attr(&f.attrs))
                    .map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            if !self.#name.lbs_omit() {
                                field_count += 1;
                            }
                        }
                    });

                // Write expressions
                let write_expressions = fields
                    .named
                    .iter()
                    .filter(|f| !has_omit_attr(&f.attrs))
                    .enumerate()
                    .map(|(i, f)| {
                        let id = get_id_attr(&f.attrs).unwrap_or(i.try_into().unwrap());
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            if !self.#name.lbs_omit() {
                                lbs::write::write_field_id(w, #id)?;
                                self.#name.lbs_write(w)?;
                            }
                        }
                    });

                // Complete body of lbs_write()
                quote! {
                    let mut field_count: u8 = 0;

                    #(#field_count_incr_expressions)*

                    lbs::write::write_field_count(w, field_count)?;

                    if field_count > 0 {
                        #(#write_expressions)*
                    }

                    Ok(())
                }
            }
            Fields::Unnamed(_) => unimplemented!(),
            Fields::Unit => quote!(Ok(())),
        },
        Data::Enum(ref data) => {
            // Ensure there is no fields with same ID
            let mut unique_ids = HashSet::new();
            for (i, v) in data.variants.iter().enumerate() {
                let id = get_id_attr(&v.attrs).unwrap_or(i.try_into().unwrap());
                if !unique_ids.insert(id) {
                    panic!("duplicate id: {}", id);
                }
            }

            // Write expressions
            let write_expressions = data.variants.iter().enumerate().map(|(i, v)| {
                let id = get_id_attr(&v.attrs).unwrap_or(i.try_into().unwrap());
                let name = &v.ident;

                if v.fields.len() > 0 {
                    return quote_spanned! {v.span()=>
                        Self::#name(inner) => {
                            lbs::write::write_field_id(w, #id)?;
                            inner.lbs_write(w)?;
                        },
                    };
                }

                quote_spanned! {v.span()=>
                    Self::#name => lbs::write::write_field_id(w, #id)?,
                }
            });

            quote! {
                match self {
                    #(#write_expressions)*
                }
                Ok(())
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

fn generate_read_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                // Field constructor expressions
                let field_constructor_expressions = fields.named.iter().map(|f| {
                    let name = &f.ident;

                    if let Some(default) = get_default_attr(&f.attrs) {
                        return quote_spanned! {f.span()=>
                            #name: #default,
                        };
                    }

                    quote_spanned! {f.span()=>
                        #name: Default::default(),
                    }
                });

                // Read expressions
                let read_expressions = fields
                    .named
                    .iter()
                    .filter(|f| !has_omit_attr(&f.attrs))
                    .enumerate()
                    .map(|(i, f)| {
                        let id = get_id_attr(&f.attrs).unwrap_or(i.try_into().unwrap());
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            #id => _self.#name = lbs::read::read(r)?,
                        }
                    });

                // Complete body of lbs_read()
                quote! {
                    let mut _self = Self {
                        #(#field_constructor_expressions)*
                    };

                    let field_count = lbs::read::read_field_count(r)?;

                    for _ in 0..field_count {
                        let id = lbs::read::read_field_id(r)?;
                        match id {
                            #(#read_expressions)*
                            _ => {},
                        }
                    }

                    Ok(_self)
                }
            }
            Fields::Unnamed(_) => unimplemented!(),
            Fields::Unit => quote!(Ok(Self)),
        },
        Data::Enum(ref data) => {
            // Read expressions
            let read_expressions = data.variants.iter().enumerate().map(|(i, v)| {
                let id = get_id_attr(&v.attrs).unwrap_or(i.try_into().unwrap());
                let name = &v.ident;

                if v.fields.len() > 0 {
                    return quote_spanned! {v.span()=>
                        #id => Ok(Self::#name(lbs::read::read(r)?)),
                    };
                }

                quote_spanned! {v.span()=>
                    #id => Ok(Self::#name),
                }
            });

            quote! {
                let id = lbs::read::read_field_id(r)?;
                match id {
                    #(#read_expressions)*
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unknown enum variant")),
                }
            }
        }
        Data::Union(_) => unimplemented!(),
    }
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

fn get_id_attr(attrs: &Vec<Attribute>) -> Option<u8> {
    for attr in attrs {
        if attr.path.is_ident(ATTR_LBS) {
            if let Ok(ident) = attr.parse_args::<syn::LitInt>() {
                return Some(ident.base10_parse().unwrap());
            }
        }
    }
    None
}

fn get_default_attr(attrs: &Vec<Attribute>) -> Option<TokenStream> {
    for attr in attrs {
        if attr.path.is_ident(ATTR_LBS_DEFAULT) {
            return Some(attr.tokens.clone());
        }
    }
    None
}

fn has_omit_attr(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path.is_ident(ATTR_LBS) {
            if let Ok(ident) = attr.parse_args::<syn::Ident>() {
                return ident.eq(FLAG_OMIT);
            }
        }
    }
    false
}

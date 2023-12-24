use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;
use std::collections::HashSet;
use syn::parenthesized;
use syn::parse::ParseBuffer;
use syn::parse_macro_input;
use syn::parse_quote;
use syn::spanned::Spanned;
use syn::Data;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Expr;
use syn::Field;
use syn::Fields;
use syn::FieldsNamed;
use syn::GenericParam;
use syn::Generics;
use syn::LitInt;
use syn::Token;
use syn::Variant;

//
// Constants.
//

const ATTRIBUTE: &str = "lbs";
const ARGUMENT_ID: &str = "id";
const ARGUMENT_DEFAULT: &str = "default";
const ARGUMENT_SKIP: &str = "skip";
const ARGUMENT_OPTIONAL: &str = "optional";

//
// Types.
//

struct Meta {
    id: Option<u16>,
    name: syn::Ident,
    default: Option<TokenStream>,
    variant_fields: Option<Fields>,
    required: bool,
    skip: bool,
    span: Span,
}

//
// Implementations.
//

impl Meta {
    fn from_struct_field(field: &Field) -> Self {
        let mut meta = Meta {
            id: None,
            name: field
                .ident
                .clone()
                .expect("unnamed fields are not supported"),
            span: field.span(),
            required: false,
            skip: false,
            default: None,
            variant_fields: None,
        };

        let mut optional = false;

        field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident(ATTRIBUTE))
            .map(|attr| {
                attr.parse_nested_meta(|arg| {
                    let arg_name = arg.path.get_ident().unwrap().to_string();

                    match arg_name.as_str() {
                        ARGUMENT_ID => {
                            let content;
                            parenthesized!(content in arg.input);
                            meta.id = Some(Self::parse_id(content));
                        }
                        ARGUMENT_DEFAULT => {
                            let content;
                            parenthesized!(content in arg.input);
                            meta.default = Some(Self::parse_default(content));
                        }
                        ARGUMENT_SKIP => meta.skip = Self::parse_flag(arg.input, ARGUMENT_SKIP),
                        ARGUMENT_OPTIONAL => {
                            optional = Self::parse_flag(arg.input, ARGUMENT_OPTIONAL)
                        }
                        unknown => panic_unknown_argument(unknown),
                    }

                    Ok(())
                })
            });

        let field_type = field.ty.to_token_stream().to_string();

        meta.required = !meta.skip
            && !optional
            && !field_type.starts_with("Option <")
            && !field_type.starts_with("core :: option :: Option <")
            && !field_type.starts_with(":: core :: option :: Option <");

        meta.validated()
    }

    fn from_enum_variant(variant: &Variant) -> Self {
        let mut meta = Meta {
            id: None,
            name: variant.ident.clone(),
            span: variant.span(),
            required: true,
            skip: false,
            default: None,
            variant_fields: if variant.fields.is_empty() {
                None
            } else {
                Some(variant.fields.clone())
            },
        };

        variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident(ATTRIBUTE))
            .map(|attr| {
                attr.parse_nested_meta(|arg| {
                    let arg_name = arg.path.get_ident().unwrap().to_string();

                    match arg_name.as_str() {
                        ARGUMENT_ID => {
                            let content;
                            parenthesized!(content in arg.input);
                            meta.id = Some(Self::parse_id(content));
                        }
                        unknown => panic_unknown_argument(unknown),
                    }

                    Ok(())
                })
            });

        meta.validated()
    }

    fn parse_id(input: ParseBuffer) -> u16 {
        input
            .parse::<LitInt>()
            .expect("id must be numeric")
            .base10_parse()
            .expect("id must fit into u16")
    }

    fn parse_default(input: ParseBuffer) -> TokenStream {
        input
            .parse::<Expr>()
            .expect("default expression expected")
            .into_token_stream()
    }

    fn parse_flag(input: &ParseBuffer, arg_name: &str) -> bool {
        if input.is_empty() || input.peek(Token![,]) {
            return true;
        }

        panic!("argument '{}' cannot have value", arg_name)
    }

    fn validated(self) -> Self {
        if self.id.is_none() {
            panic!(
                "struct field or enum variant must have an id: #[{}({}(<u16>))]",
                ATTRIBUTE, ARGUMENT_ID
            )
        }

        self
    }
}

//
// Derive LBSWrite.
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
        Data::Enum(ref data) => generate_write_body_for_enum(data),
        Data::Union(_) => panic!("unions are unsupported"),
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => generate_write_body_for_struct(fields),
            Fields::Unnamed(_) => panic!("structs with unnamed fields are unsupported"),
            Fields::Unit => quote!(Ok(())),
        },
    };

    // Complete trait implementation
    proc_macro::TokenStream::from(quote! {
        impl #impl_generics lbs::LBSWrite for #name #ty_generics #where_clause {
            #[inline]
            fn lbs_write<W: std::io::Write>(&self, w: &mut W) -> core::result::Result<(), lbs::error::LBSError> {
                #write_body
            }
        }
    })
}

//
// Derive LBSRead.
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
            fn lbs_read<R: std::io::Read>(r: &mut R) -> core::result::Result<Self, lbs::error::LBSError> {
                #read_body
            }
        }
    })
}

fn generate_write_body_for_struct(fields: &FieldsNamed) -> TokenStream {
    // Gather meta
    let meta = gather_struct_meta(fields);

    // Field count expressions
    let field_count_expressions = meta.iter().filter(|m| !m.skip).map(|m| {
        let field_name = &m.name;
        quote_spanned! {m.span=>
            if self.#field_name.lbs_must_write() {
                field_count += 1;
            }
        }
    });

    // Write expressions
    let write_expressions = meta.iter().filter(|m| !m.skip).map(|m| {
        let field_id = m.id;
        let field_name = &m.name;
        quote_spanned! {m.span=>
            if self.#field_name.lbs_must_write() {
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

        if m.variant_fields.is_some() {
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
    // Gather meta.
    let meta = gather_struct_meta(fields);

    // Field initialization expressions.
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

    // Required fields stuff.
    let required_count = meta.iter().filter(|f| f.required).count();
    let mut required_index_read = 0usize;
    let mut required_index_check = 0usize;

    // Read expressions.
    let read_expressions = meta.iter().filter(|f| !f.skip).map(|f| {
        let field_id = f.id;
        let field_name = &f.name;

        let expr = if f.required {
            quote_spanned! {f.span=>
                #field_id => {
                    _self.#field_name = lbs::read::read(r).map_err(|e| e.with_field(#field_id))?;
                    required_present[#required_index_read] = true;
                }
            }
        } else {
            quote_spanned! {f.span=>
                #field_id => _self.#field_name = lbs::read::read(r).map_err(|e| e.with_field(#field_id))?,
            }
        };

        if f.required {
            required_index_read += 1;
        }

        expr
    });

    // Required check expressions.
    let required_check_expressions = meta.iter().filter(|f| f.required).map(|f| {
        let field_id = f.id;

        let expr = quote_spanned! {f.span=>
            if !required_present[#required_index_check] {
                return Err(lbs::error::LBSError::RequiredButMissing.with_field(#field_id));
            }
        };

        required_index_check += 1;
        expr
    });

    // Complete body of lbs_read().
    quote! {
        let mut _self = Self {
            #(#field_init_expressions)*
        };

        let mut required_present = [false; #required_count];

        for _ in 0..lbs::read::read_field_count(r)? {
            match lbs::read::read_field_id(r)? {
                #(#read_expressions)*
                _ => {},
            }
        }

        #(#required_check_expressions)*

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

        if m.variant_fields.is_some() {
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
            _ => Err(lbs::error::LBSError::UnexpectedVariant)
        }
    }
}

fn gather_struct_meta(fields: &FieldsNamed) -> Vec<Meta> {
    let mut metas = Vec::new();
    let mut unique_ids = HashSet::new();

    for field in &fields.named {
        let meta = Meta::from_struct_field(field);
        let id = meta.id.unwrap();

        if !unique_ids.insert(id) {
            panic_duplicated_id(id);
        }

        metas.push(meta);
    }

    metas
}

fn gather_enum_meta(data: &DataEnum) -> Vec<Meta> {
    let mut metas = Vec::new();
    let mut unique_ids = HashSet::new();

    for variant in &data.variants {
        if variant.fields.len() > 1 {
            panic!("unsupported enum variant");
        }

        match variant.fields {
            Fields::Unit => {}
            Fields::Unnamed(_) => {}
            _ => panic!("unsupported enum variant"),
        }

        let meta = Meta::from_enum_variant(variant);
        let id = meta.id.unwrap();

        if !unique_ids.insert(id) {
            panic_duplicated_id(id);
        }

        metas.push(meta);
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

fn panic_duplicated_id(id: u16) {
    panic!("duplicated id {}", id);
}

fn panic_unknown_argument(name: &str) {
    panic!("unknown argument '{}'", name)
}

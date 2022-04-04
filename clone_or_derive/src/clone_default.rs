use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Path, Type, TypePath};

pub fn impl_hello_world(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let field_token_stream = fields
        .into_iter()
        .map(|f| {
            // Interpolation only works for variables, not arbitrary expressions.
            // That's why we need to move these fields into local variables first
            // (borrowing would also work though).
            let field_name = &f.ident;
            let field_ty = &f.ty;
            let mut nested = false;
            for attr in f.attrs.iter() {
                match attr.parse_meta().unwrap() {
                    // Find the duplicated idents
                    syn::Meta::Path(ref path)
                        if path.get_ident().unwrap().to_string() == "clone_or" =>
                    {
                        nested = true;
                    }
                    _ => (),
                }
            }

            match (nested, is_option(&field_ty)) {
                (true, true) => Ok(
                    quote! { #field_name : match (self.#field_name.as_ref(), default.#field_name.as_ref())
                                {
                                    (Some(p),Some(q)) => Some(p.clone_or(&q)),
                                    (Some(p),None) => Some(p.clone()),
                                    (None,Some(p)) => Some(p.clone()),
                                    (None,None) => None,
                                }, },
                ),
                (false, true) => Ok(
                    quote! {#field_name : match self.#field_name
                                .as_ref()
                                .or(default.#field_name.as_ref())
                            {
                                Some(p) => Some(p.clone()),
                                None => None,
                            }, }),
                (true, false) => Ok(
                    quote! { #field_name : self.#field_name.clone_or(&default.#field_name), }),
                (false, false) => Ok(quote! {#field_name :  self.#field_name.clone(), }),
            }
        })
        .collect::<syn::Result<TokenStream>>()?;

    Ok(quote! {
    #[automatically_derived]
    impl CloneOr for #name {
        fn clone_or(&self, default: & #name) -> Self {
            #name {
                #field_token_stream
            }
            }
        }
    })
}

fn is_option(field_ty: &Type) -> bool {
    match field_ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => {
            // segments is of Type syn::punctuated::Punctuated<PathSegment, _>
            if let Some(path_seg) = segments.first() {
                let ident = &path_seg.ident;
                return ident == "Option";
            }
            false
        }
        _ => false,
    }
}
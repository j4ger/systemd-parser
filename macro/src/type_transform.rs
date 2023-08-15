use syn::{Error, GenericArgument, Path, PathArguments, PathSegment, Type};

pub(crate) fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(typepath) if typepath.qself.is_none() => {
            let idents_of_path = typepath
                .path
                .segments
                .iter()
                .fold(String::new(), |mut acc, v| {
                    acc.push_str(&v.ident.to_string());
                    acc.push(':');
                    acc
                });
            vec!["Option:", "std:option:Option:", "core:option:Option:"]
                .into_iter()
                .find(|s| idents_of_path == *s)
                .and_then(|_| typepath.path.segments.last())
                .is_some()
        }
        _ => false,
    }
}

pub(crate) fn is_vec(ty: &Type) -> bool {
    match ty {
        Type::Path(typepath) if typepath.qself.is_none() => {
            let idents_of_path = typepath
                .path
                .segments
                .iter()
                .fold(String::new(), |mut acc, v| {
                    acc.push_str(&v.ident.to_string());
                    acc.push(':');
                    acc
                });
            vec!["Vec:", "std:vec:Vec:", "alloc:vec:Vec:"]
                .into_iter()
                .find(|s| idents_of_path == *s)
                .and_then(|_| typepath.path.segments.last())
                .is_some()
        }
        _ => false,
    }
}

// credits: https://stackoverflow.com/a/56264023
fn extract_type_from_option(ty: &syn::Type) -> Result<&Type, Error> {
    fn extract_type_path(ty: &syn::Type) -> Result<&Path, Error> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Ok(&typepath.path),
            _ => Err(Error::new_spanned(ty, "Expected `Option<T>`.")),
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_option_segment(path: &Path) -> Result<&PathSegment, Error> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
            .ok_or(Error::new_spanned(path, "Expected `Option<T>.`"))
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params
                    .args
                    .first()
                    .ok_or(Error::new_spanned(ty, "Expected `Option<T>.`")),
                _ => Err(Error::new_spanned(ty, "Expected `Option<T>.`")),
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Ok(ty),
            _ => Err(Error::new_spanned(ty, "Expected `Option<T>.`")),
        })
}

fn extract_type_from_vec(ty: &syn::Type) -> Result<&Type, Error> {
    fn extract_type_path(ty: &syn::Type) -> Result<&Path, Error> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Ok(&typepath.path),
            _ => Err(Error::new_spanned(ty, "Expected `Vec<T>`.")),
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_vec_segment(path: &Path) -> Result<&PathSegment, Error> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Vec|", "std|vec|Vec|", "alloc|vec|Vec|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
            .ok_or(Error::new_spanned(path, "Expected `Vec<T>.`"))
    }

    extract_type_path(ty)
        .and_then(|path| extract_vec_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params
                    .args
                    .first()
                    .ok_or(Error::new_spanned(ty, "Expected `Vec<T>.`")),
                _ => Err(Error::new_spanned(ty, "Expected `Vec<T>.`")),
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Ok(ty),
            _ => Err(Error::new_spanned(ty, "Expected `Vec<T>.`")),
        })
}

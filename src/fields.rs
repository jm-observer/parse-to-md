use anyhow::Result;
use log::{debug, warn};
use syn::{
    Field as SynField, GenericArgument, PathArguments, Type,
    __private::ToTokens
};
#[derive(Debug)]
pub struct ParseField {
    pub name:  String,
    pub attrs: Vec<String>,
    pub ty:    String,
    pub objs:  Vec<String>
}

impl ParseField {
    pub fn try_from(field: &SynField) -> Self {
        let name = field
            .ident
            .clone()
            .map(|x| x.to_string())
            .unwrap_or_default();
        let attrs = field
            .attrs
            .iter()
            .map(|x| {
                x.meta.to_token_stream().to_string().replace(" ", "")
            })
            .collect();
        let ty =
            field.ty.to_token_stream().to_string().replace(" ", "");
        let mut objs = Vec::new();
        get_tys(&field.ty, &mut objs);
        Self {
            name,
            attrs,
            ty,
            objs
        }
    }
}

pub fn get_tys(ty: &Type, tys: &mut Vec<String>) {
    match ty {
        Type::Path(path) => {
            for segment in &path.path.segments {
                let ident = segment.ident.to_string();
                if match_ty(ident.as_str()) {
                    tys.push(ident);
                }
                match &segment.arguments {
                    PathArguments::None => {},
                    PathArguments::AngleBracketed(conf) => {
                        for x in &conf.args {
                            match x {
                                GenericArgument::Type(ty) => {
                                    get_tys(ty, tys)
                                },
                                _ => {
                                    warn!(
                                        "not support {}",
                                        x.to_token_stream()
                                            .to_string()
                                    );
                                }
                            }
                        }
                    },
                    PathArguments::Parenthesized(conf) => {
                        warn!(
                            "not support {}",
                            conf.to_token_stream().to_string()
                        );
                    }
                }
            }
        },
        _ => {
            warn!("{}", ty.to_token_stream().to_string())
        }
    }
}

fn match_ty(ty: &str) -> bool {
    match ty {
        "Option" | "Vec" | "Arc" => false,
        _ => true
    }
}

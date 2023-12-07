use anyhow::{anyhow, bail, Result};
use async_recursion::async_recursion;
use log::{debug, error, warn};
use rust_lib::{
    fields::ParseField, regexs::Regexs, serdes::Rename,
    variants::ParseVariant
};
use std::{ops::Add, path::PathBuf, sync::Arc};
use syn::{
    spanned::Spanned,
    Item, ItemEnum, ItemStruct, Meta, PathArguments, Type,
    __private::{quote::__private::ext::RepToTokensExt, ToTokens}
};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    custom_utils::logger::logger_stdout_debug();
    let rx = Regexs::init()?;

    let path = "D:\\git\\iiotdaa\\src\\lib.rs";

    let items =
        get_item(ModPath::Lib { path: path.into() }, &rx).await?;
    for item in items {
        match item {
            ItemData::Enum {
                item,
                local,
                rename_all,
                tag,
                content,
                name,
                variants
            } => {
                if name.as_str() == "DevProtocolType"
                    || name.as_str() == "BatchAcquisition"
                {
                    for var in variants {
                        debug!("{:?}", var);
                    }
                }
            },
            ItemData::Struct {
                rename_all,
                local,
                fields,
                name
            } => {
                // if name.as_str() == "UploadDataBuilder" {
                //     debug!("{} {:?} ", local.mod_path(),
                // rename_all);     debug!("{:?} ",
                // fields); }
            }
        }
    }
    Ok(())
}
#[async_recursion(?Send)]
async fn get_item(
    path: ModPath,
    rx: &Regexs
) -> Result<Vec<ItemData>> {
    let file_content = fs::read_to_string(path.path()).await?;
    let file = syn::parse_file(file_content.as_str())?;
    let arc_path = Arc::new(path);
    let mut datas = Vec::new();
    for item in file.items {
        match item {
            Item::Enum(enum_item) => {
                let mut rename_all = None;
                let mut tag = None;
                let mut content = None;
                for attr in enum_item.attrs.iter() {
                    let Meta::List(list) = &attr.meta else {
                        continue;
                    };
                    if list
                        .path
                        .segments
                        .iter()
                        .find(|x| {
                            x.ident.to_string().as_str() == "serde"
                        })
                        .is_some()
                    {
                        let mut iter =
                            list.tokens.clone().into_iter();
                        while let Some(name) = iter.next() {
                            let name = name.to_string();
                            match name.as_str() {
                                "rename_all" => {
                                    let _ = iter.next();
                                    let Some(val) = iter.next()
                                    else {
                                        bail!("rename_all is empty");
                                    };
                                    rename_all =
                                        Some(Rename::from_str(
                                            val.to_string().as_str()
                                        )?);
                                },
                                "tag" => {
                                    let _ = iter.next();
                                    let Some(val) = iter.next()
                                    else {
                                        bail!("tag is empty");
                                    };
                                    tag = Some(val.to_string());
                                },
                                "content" => {
                                    let _ = iter.next();
                                    let Some(val) = iter.next()
                                    else {
                                        bail!("content is empty");
                                    };
                                    content = Some(val.to_string());
                                },
                                "try_from" | "into" | "from" => {
                                    let _ = iter.next();
                                    let _ = iter.next();
                                },
                                "," | "deny_unknown_fields" => {},
                                _ => {
                                    warn!(
                                        "serde enum not support {} \
                                         {}",
                                        name,
                                        enum_item.ident.to_string()
                                    );
                                }
                            }
                        }
                    } else {
                        continue;
                    }
                }
                let variants = enum_item
                    .variants
                    .iter()
                    .map(|x| ParseVariant::from(x))
                    .collect();
                let name = enum_item.ident.to_string();
                let data = ItemData::Enum {
                    item: enum_item,
                    local: arc_path.clone(),
                    rename_all,
                    tag,
                    content,
                    name,
                    variants
                };
                datas.push(data);
            },
            Item::Mod(mod_item) => {
                let mod_name = mod_item.ident.to_string();
                if mod_name.starts_with("test") {
                    continue;
                }
                let mod_path =
                    get_mod_path_v2(&arc_path, mod_name.as_str())
                        .await?;
                datas.extend(get_item(mod_path, rx).await?);
            },
            Item::Struct(struct_item) => {
                let mut rename_all = None;
                for attr in struct_item.attrs.iter() {
                    let Meta::List(list) = &attr.meta else {
                        continue;
                    };
                    if list
                        .path
                        .segments
                        .iter()
                        .find(|x| {
                            x.ident.to_string().as_str() == "serde"
                        })
                        .is_some()
                    {
                        let mut iter =
                            list.tokens.clone().into_iter();
                        while let Some(name) = iter.next() {
                            let name = name.to_string();
                            match name.as_str() {
                                "rename_all" => {
                                    let _ = iter.next();
                                    let Some(val) = iter.next()
                                    else {
                                        bail!("rename_all is empty");
                                    };
                                    rename_all =
                                        Some(Rename::from_str(
                                            val.to_string().as_str()
                                        )?);
                                },
                                "try_from" | "into" | "from" => {
                                    let _ = iter.next();
                                    let _ = iter.next();
                                },
                                "," | "deny_unknown_fields" => {},
                                _ => {
                                    warn!(
                                        "serde struct not support \
                                         {} {}",
                                        name,
                                        struct_item.ident.to_string()
                                    );
                                }
                            }
                        }
                    } else {
                        continue;
                    }
                }

                let mut fields =
                    Vec::with_capacity(struct_item.fields.len());
                let name = struct_item.ident.to_string();
                if name.as_str() == "UploadDataBuilder" {
                    debug!("{:?} ", name);
                }
                for field in &struct_item.fields {
                    fields.push(ParseField::try_from(field));
                }
                let data = ItemData::Struct {
                    fields,
                    local: arc_path.clone(),
                    rename_all,
                    name
                };
                datas.push(data);
            },
            Item::Const(_)
            | Item::ExternCrate(_)
            | Item::Fn(_)
            | Item::ForeignMod(_)
            | Item::Impl(_)
            | Item::Macro(_)
            | Item::Static(_)
            | Item::Trait(_)
            | Item::TraitAlias(_)
            | Item::Type(_)
            | Item::Union(_)
            | Item::Use(_)
            | Item::Verbatim(_) => {},
            _ => {}
        }
    }
    Ok(datas)
}

#[derive(Debug)]
enum ItemData {
    Enum {
        item:       ItemEnum,
        local:      Arc<ModPath>,
        rename_all: Option<Rename>,
        tag:        Option<String>,
        content:    Option<String>,
        name:       String,
        variants:   Vec<ParseVariant>
    },
    Struct {
        name:       String,
        rename_all: Option<Rename>,
        local:      Arc<ModPath>,
        fields:     Vec<ParseField>
    }
}

// impl From<(ItemEnum, Arc<ModPath>)> for ItemData {
//     fn from(value: (ItemEnum, Arc<ModPath>)) -> Self {
//         Self::Enum {
//             item:  value.0,
//             local: value.1
//         }
//     }
// }
// impl From<(ItemStruct, Arc<ModPath>)> for ItemData {
//     fn from(value: (ItemStruct, Arc<ModPath>)) -> Self {
//         Self::Struct {
//             item:       value.0,
//             rename_all: None,
//             local:      value.1
//         }
//     }
// }

#[derive(Debug, Clone)]
enum ModPath {
    Lib {
        path: PathBuf
    },
    /// mod.rs
    Mod {
        name:     String,
        path:     PathBuf,
        mod_path: String
    },
    /// gateway.rs
    Name {
        name:     String,
        path:     PathBuf,
        mod_path: String
    }
}

impl ModPath {
    pub fn path(&self) -> &PathBuf {
        match self {
            ModPath::Lib { path } => path,
            ModPath::Mod { path, .. } => path,
            ModPath::Name { path, .. } => path
        }
    }

    pub fn mod_path(&self) -> &str {
        match self {
            ModPath::Lib { .. } => "crate",
            ModPath::Mod { mod_path, .. } => mod_path.as_str(),
            ModPath::Name { mod_path, .. } => mod_path.as_str()
        }
    }
}

async fn get_mod_path_v2(
    mod_path: &ModPath,
    sub_mod_name: &str
) -> Result<ModPath> {
    match mod_path {
        ModPath::Lib { path } | ModPath::Mod { path, .. } => {
            let parent_path = path.parent().ok_or(anyhow!(
                "get parent path fail: {:?}",
                mod_path
            ))?;
            if parent_path
                .join(format!("{}.rs", sub_mod_name))
                .exists()
            {
                let sub = ModPath::Name {
                    name:     sub_mod_name.to_string(),
                    path:     parent_path
                        .join(format!("{}.rs", sub_mod_name)),
                    mod_path: mod_path
                        .mod_path()
                        .to_string()
                        .add("::")
                        .add(sub_mod_name)
                };
                return Ok(sub);
            } else if parent_path
                .join(sub_mod_name)
                .join("mod.rs")
                .exists()
            {
                let sub = ModPath::Mod {
                    name:     sub_mod_name.to_string(),
                    path:     parent_path
                        .join(sub_mod_name)
                        .join("mod.rs"),
                    mod_path: mod_path
                        .mod_path()
                        .to_string()
                        .add("::")
                        .add(sub_mod_name)
                };
                return Ok(sub);
            } else {
                bail!(
                    "could not find mod's path: {:?} {}",
                    mod_path,
                    sub_mod_name
                );
            }
        },
        ModPath::Name { path, name, .. } => {
            let parent_path = path
                .parent()
                .ok_or(anyhow!(
                    "get parent path fail: {:?}",
                    mod_path
                ))?
                .join(name);
            if parent_path
                .join(format!("{}.rs", sub_mod_name))
                .exists()
            {
                let sub = ModPath::Name {
                    name:     sub_mod_name.to_string(),
                    path:     parent_path
                        .join(format!("{}.rs", sub_mod_name)),
                    mod_path: mod_path
                        .mod_path()
                        .to_string()
                        .add("::")
                        .add(sub_mod_name)
                };
                return Ok(sub);
            } else if parent_path
                .join(sub_mod_name)
                .join("mod.rs")
                .exists()
            {
                let sub = ModPath::Mod {
                    name:     sub_mod_name.to_string(),
                    path:     parent_path
                        .join(sub_mod_name)
                        .join("mod.rs"),
                    mod_path: mod_path
                        .mod_path()
                        .to_string()
                        .add("::")
                        .add(sub_mod_name)
                };
                return Ok(sub);
            } else {
                bail!(
                    "could not find mod's path: {:?} {}",
                    mod_path,
                    sub_mod_name
                );
            }
        }
    }
}

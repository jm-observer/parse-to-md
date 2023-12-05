use anyhow::{anyhow, bail, Result};
use async_recursion::async_recursion;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use syn::{Item, ItemEnum, ItemStruct};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let path = "D:\\git\\iiotdaa\\src\\lib.rs";

    let items = get_item(ModPath::Lib { path: path.into() }).await?;
    for item in items {
        match item {
            ItemData::Enum { .. } => {}
            ItemData::Struct { item, local } => {
                if item.ident.to_string().as_str() == "UploadData" {
                    println!("{} {:?} ", local.mod_path(), item);
                }
            }
        }
    }
    Ok(())
}
#[async_recursion(?Send)]
async fn get_item(path: ModPath) -> Result<Vec<ItemData>> {
    let file_content = fs::read_to_string(path.path()).await?;
    let file = syn::parse_file(file_content.as_str())?;
    let arc_path = Arc::new(path);
    let mut datas = Vec::new();
    for item in file.items {
        match item {
            Item::Enum(enum_item) => {
                datas.push((enum_item, arc_path.clone()).into());
            }
            Item::Mod(mod_item) => {
                let mod_name = mod_item.ident.to_string();
                if mod_name.starts_with("test") {
                    continue;
                }
                let mod_path = get_mod_path_v2(&arc_path, mod_name.as_str()).await?;
                datas.extend(get_item(mod_path).await?);
            }
            Item::Struct(struct_item) => {
                datas.push((struct_item, arc_path.clone()).into());
            }
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
            | Item::Verbatim(_) => {}
            _ => {}
        }
    }
    Ok(datas)
}

#[derive(Debug)]
enum ItemData {
    Enum {
        item: ItemEnum,
        local: Arc<ModPath>,
    },
    Struct {
        item: ItemStruct,
        local: Arc<ModPath>,
    },
}

impl From<(ItemEnum, Arc<ModPath>)> for ItemData {
    fn from(value: (ItemEnum, Arc<ModPath>)) -> Self {
        Self::Enum {
            item: value.0,
            local: value.1,
        }
    }
}
impl From<(ItemStruct, Arc<ModPath>)> for ItemData {
    fn from(value: (ItemStruct, Arc<ModPath>)) -> Self {
        Self::Struct {
            item: value.0,
            local: value.1,
        }
    }
}

#[derive(Debug, Clone)]
enum ModPath {
    Lib {
        path: PathBuf,
    },
    /// mod.rs
    Mod {
        name: String,
        path: PathBuf,
        mod_path: String,
    },
    /// gateway.rs
    Name {
        name: String,
        path: PathBuf,
        mod_path: String,
    },
}

impl ModPath {
    pub fn path(&self) -> &PathBuf {
        match self {
            ModPath::Lib { path } => path,
            ModPath::Mod { path, .. } => path,
            ModPath::Name { path, .. } => path,
        }
    }
    pub fn mod_path(&self) -> &str {
        match self {
            ModPath::Lib { .. } => "crate",
            ModPath::Mod { mod_path, .. } => mod_path.as_str(),
            ModPath::Name { mod_path, .. } => mod_path.as_str(),
        }
    }
}

async fn get_mod_path_v2(mod_path: &ModPath, sub_mod_name: &str) -> Result<ModPath> {
    match mod_path {
        ModPath::Lib { path } | ModPath::Mod { path, .. } => {
            let parent_path = path
                .parent()
                .ok_or(anyhow!("get parent path fail: {:?}", mod_path))?;
            if parent_path.join(format!("{}.rs", sub_mod_name)).exists() {
                let sub = ModPath::Name {
                    name: sub_mod_name.to_string(),
                    path: parent_path.join(format!("{}.rs", sub_mod_name)),
                    mod_path: mod_path.mod_path().to_string().add("::").add(sub_mod_name),
                };
                return Ok(sub);
            } else if parent_path.join(sub_mod_name).join("mod.rs").exists() {
                let sub = ModPath::Mod {
                    name: sub_mod_name.to_string(),
                    path: parent_path.join(sub_mod_name).join("mod.rs"),
                    mod_path: mod_path.mod_path().to_string().add("::").add(sub_mod_name),
                };
                return Ok(sub);
            } else {
                bail!("could not find mod's path: {:?} {}", mod_path, sub_mod_name);
            }
        }
        ModPath::Name { path, name, .. } => {
            let parent_path = path
                .parent()
                .ok_or(anyhow!("get parent path fail: {:?}", mod_path))?
                .join(name);
            if parent_path.join(format!("{}.rs", sub_mod_name)).exists() {
                let sub = ModPath::Name {
                    name: sub_mod_name.to_string(),
                    path: parent_path.join(format!("{}.rs", sub_mod_name)),
                    mod_path: mod_path.mod_path().to_string().add("::").add(sub_mod_name),
                };
                return Ok(sub);
            } else if parent_path.join(sub_mod_name).join("mod.rs").exists() {
                let sub = ModPath::Mod {
                    name: sub_mod_name.to_string(),
                    path: parent_path.join(sub_mod_name).join("mod.rs"),
                    mod_path: mod_path.mod_path().to_string().add("::").add(sub_mod_name),
                };
                return Ok(sub);
            } else {
                bail!("could not find mod's path: {:?} {}", mod_path, sub_mod_name);
            }
        }
    }
}

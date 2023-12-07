use crate::fields::ParseField;
use syn::Variant;

#[derive(Debug)]
pub struct ParseVariant {
    /// Name of the variant.
    pub name:   String,
    /// Content stored in the variant.
    pub fields: Vec<ParseField>
}

impl ParseVariant {
    pub fn from(var: &Variant) -> Self {
        let Variant {
            attrs,
            ident,
            fields,
            discriminant
        } = var;
        let fields =
            fields.iter().map(|x| ParseField::try_from(x)).collect();
        Self {
            name: ident.to_string(),
            fields
        }
    }
}

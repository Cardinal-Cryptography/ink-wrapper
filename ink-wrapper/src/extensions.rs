use ink_metadata::MessageSpec;
use scale_info::{form::PortableForm, Field, Type, TypeDefComposite, Variant};

pub trait TypeExtensions {
    /// Returns true if the type is a rust primitive.
    fn is_primitive(&self) -> bool;

    /// Returns true if the type is defined in the ink_primitives crate.
    fn is_ink(&self) -> bool;

    /// Returns true if the type is defined in the ink_primitives crate's private types module. These types are
    /// reexported at the top level in ink_primitives.
    fn is_ink_types(&self) -> bool;

    /// Returns true if the type is a builtin type.
    fn is_builtin(&self) -> bool;

    /// Returns true if the type is defined in the contract itself.
    fn is_custom(&self) -> bool;

    /// Returns true if the type is exactly the LangError type. We wrap this type into our own, because it's not
    /// `Error`.
    fn is_lang_error(&self) -> bool;

    /// Returns the name by which the type can be referenced.
    ///
    /// It's the full path to the type for ink! types and just the name for other types. That's because any custom types
    /// for the contract will be defined in the same module as the functions that use them.
    fn qualified_name(&self) -> String;
}

impl TypeExtensions for Type<PortableForm> {
    fn is_primitive(&self) -> bool {
        matches!(self.type_def(), scale_info::TypeDef::Primitive(_))
    }

    fn is_ink(&self) -> bool {
        !self.path().segments().is_empty() && self.path().segments()[0] == "ink_primitives"
    }

    fn is_ink_types(&self) -> bool {
        self.path().segments().len() > 2
            && self.path().segments()[0] == "ink_primitives"
            && self.path().segments()[1] == "types"
    }

    fn is_builtin(&self) -> bool {
        self.path().segments().len() == 1
    }

    fn is_lang_error(&self) -> bool {
        self.is_ink() && self.path().segments().last().unwrap() == "LangError"
    }

    fn is_custom(&self) -> bool {
        !self.is_primitive() && !self.is_ink() && !self.is_builtin()
    }

    fn qualified_name(&self) -> String {
        if self.is_lang_error() {
            "ink_wrapper_types::InkLangError".to_string()
        } else if self.is_ink_types() {
            ["ink_primitives", self.path().segments().last().unwrap()].join("::")
        } else if self.is_ink() {
            self.path().segments().join("::")
        } else {
            self.path().segments().last().unwrap().to_string()
        }
    }
}

pub trait MessageSpecExtensions {
    fn trait_name(&self) -> Option<String>;
    fn method_name(&self) -> String;
}

impl MessageSpecExtensions for MessageSpec<PortableForm> {
    fn trait_name(&self) -> Option<String> {
        let parts = self.label().split("::").collect::<Vec<&str>>();
        match parts.len() {
            1 => None,
            2 => Some(parts[0].to_string()),
            _ => panic!(
                "Nested modules in method names are unsupported yet: {}",
                self.label()
            ),
        }
    }

    fn method_name(&self) -> String {
        self.label().split("::").last().unwrap().to_string()
    }
}

/// A type describing the fields of a struct or enum.
///
/// The typing on TypeDef does not guarantee that all fields are either named or unnamed, so we convert to this type
/// first.
pub enum Fields {
    /// A type with named fields.
    Named(Vec<(String, u32)>),
    /// A type with unnamed fields.
    Unnamed(Vec<u32>),
}

impl From<Vec<&Field<PortableForm>>> for Fields {
    fn from(fields: Vec<&Field<PortableForm>>) -> Self {
        if fields.iter().all(|f| f.name().is_none()) {
            Fields::Unnamed(fields.iter().map(|f| f.ty().id()).collect())
        } else {
            Fields::Named(
                fields
                    .iter()
                    .map(|f| {
                        (
                            f.name()
                                .unwrap_or_else(|| {
                                    panic!("{:?} has a mix of named and unnamed fields", fields)
                                })
                                .to_string(),
                            f.ty().id(),
                        )
                    })
                    .collect(),
            )
        }
    }
}

/// An extension trait that allows extraction of a [Fields] from the implementor.
pub trait AggregateFields {
    fn aggregate_fields(&self) -> Fields;
}

impl AggregateFields for Variant<PortableForm> {
    fn aggregate_fields(&self) -> Fields {
        self.fields()
            .iter()
            .collect::<Vec<&Field<PortableForm>>>()
            .into()
    }
}

impl AggregateFields for TypeDefComposite<PortableForm> {
    fn aggregate_fields(&self) -> Fields {
        self.fields()
            .iter()
            .collect::<Vec<&Field<PortableForm>>>()
            .into()
    }
}

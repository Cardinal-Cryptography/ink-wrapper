use anyhow::Result;
use clap::Parser;
use genco::fmt;
use genco::prelude::*;
use ink_metadata::{ConstructorSpec, InkProject, MessageParamSpec, MessageSpec};
use scale_info::TypeDefPrimitive;
use scale_info::{
    form::PortableForm, Field, Type, TypeDef, TypeDefArray, TypeDefCompact, TypeDefComposite,
    TypeDefSequence, TypeDefTuple, TypeDefVariant, Variant,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser)]
struct Args {
    #[arg(
        short,
        long,
        help = "Path to the metadata file to generate a wrapper for."
    )]
    metadata: String,
}

/// Struct for deserializing metadata.json that contains the fields not present in an InkProject.
#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    source: Source,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    hash: String,
}

trait TypeExtensions {
    fn is_primitive(&self) -> bool;
    fn is_ink(&self) -> bool;
    fn is_ink_types(&self) -> bool;
    fn is_builtin(&self) -> bool;
    fn qualified_name(&self) -> String;
}

impl TypeExtensions for Type<PortableForm> {
    /// Returns true if the type is a rust primitive.
    fn is_primitive(&self) -> bool {
        matches!(self.type_def(), scale_info::TypeDef::Primitive(_))
    }

    /// Returns true if the type is defined in the ink_primitives crate.
    fn is_ink(&self) -> bool {
        !self.path().segments().is_empty() && self.path().segments()[0] == "ink_primitives"
    }

    /// Returns true if the type is defined in the ink_primitives crate's private types module. These types are
    /// reexported at the top level in ink_primitives.
    fn is_ink_types(&self) -> bool {
        self.path().segments().len() > 2
            && self.path().segments()[0] == "ink_primitives"
            && self.path().segments()[1] == "types"
    }

    /// Returns true if the type is a builtin type.
    fn is_builtin(&self) -> bool {
        self.path().segments().len() == 1
    }

    /// Returns the name by which the type can be referenced.
    ///
    /// It's the full path to the type for ink! types and just the name for other types. That's because any custom types
    /// for the contract will be defined in the same module as the functions that use them.
    fn qualified_name(&self) -> String {
        if self.is_ink_types() {
            ["ink_primitives", self.path().segments().last().unwrap()].join("::")
        } else if self.is_ink() {
            self.path().segments().join("::")
        } else {
            self.path().segments().last().unwrap().to_string()
        }
    }
}

/// A type describing the fields of a struct or enum.
///
/// The typing on TypeDef does not guarantee that all fields are either named or unnamed, so we convert to this type
/// first.
enum Fields {
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
trait AggregateFields {
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

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex.replace("0x", "")).unwrap()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let jsonized = fs::read_to_string(args.metadata)?;
    let metadata: Metadata = serde_json::from_str(&jsonized)?;
    let code_hash = metadata.source.hash;
    let metadata: InkProject = serde_json::from_str(&jsonized)?;

    let tokens: rust::Tokens = generate(&metadata, code_hash);

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(4));

    let config = rust::Config::default().with_default_import(rust::ImportMode::Qualified);

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}

/// Generates the full wrapper for the contract.
fn generate(metadata: &InkProject, code_hash: String) -> rust::Tokens {
    let encode = rust::import("scale", "Encode").with_alias("_");

    quote! {
        $(register(encode))

        $(for typ in metadata.registry().types() {
            $(if !typ.ty().is_primitive() && !typ.ty().is_ink() && !typ.ty().is_builtin() {
                $(define_type(typ.ty(), metadata))
            })
        })

        #[derive(Debug, Clone, Copy)]
        pub struct Instance {
            account_id: ink_primitives::AccountId,
        }

        impl From<ink_primitives::AccountId> for Instance {
            fn from(account_id: ink_primitives::AccountId) -> Self {
                Self { account_id }
            }
        }

        impl From<Instance> for ink_primitives::AccountId {
            fn from(instance: Instance) -> Self {
                instance.account_id
            }
        }

        impl Instance {
            $(for constructor in metadata.spec().constructors().iter() {
                $(define_constructor(&code_hash, constructor, metadata)) $['\n']
            })

            $(for message in metadata.spec().messages() {
                $(define_message(message, metadata))
            })
        }
    }
}

/// Generates a type definition for a custom type used in the contract.
fn define_type(typ: &Type<PortableForm>, metadata: &InkProject) -> rust::Tokens {
    match &typ.type_def() {
        TypeDef::Variant(variant) => define_variant(typ, variant, metadata),
        TypeDef::Composite(composite) => define_composite(typ, composite, metadata),
        _ => quote! {},
    }
}

/// Generates a type definition for an enum.
fn define_variant(
    typ: &Type<PortableForm>,
    variant: &TypeDefVariant<PortableForm>,
    metadata: &InkProject,
) -> rust::Tokens {
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
        pub enum $(typ.qualified_name()) {
            $(for variant in variant.variants() {
                $(match variant.aggregate_fields() {
                    Fields::Named(fields) => {
                        $(&variant.name) {
                            $(for (name, typ) in fields {
                                $(name): $(type_ref(typ, metadata)),
                            })
                        },
                    },
                    Fields::Unnamed(fields) => {
                        $(&variant.name) (
                            $(for typ in fields {
                                $(type_ref(typ, metadata)),
                            })
                        ),
                    },
                })
            })
        }
        $[ '\n' ]
    }
}

/// Generates a type definition for a struct.
fn define_composite(
    typ: &Type<PortableForm>,
    composite: &TypeDefComposite<PortableForm>,
    metadata: &InkProject,
) -> rust::Tokens {
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
        $(match composite.aggregate_fields() {
            Fields::Named(fields) => {
                pub struct $(typ.qualified_name()) {
                    $(for (name, typ) in fields {
                        pub $(name): $(type_ref(typ, metadata)),
                    })
                }
            },

            Fields::Unnamed(fields) => {
                pub struct $(typ.qualified_name()) (
                    $(for typ in fields {
                        pub $(type_ref(typ, metadata)),
                    })
                );
            },
        })

        $[ '\n' ]
    }
}

/// Generates a function wrapping a contract constructor.
fn define_constructor(
    code_hash: &str,
    constructor: &ConstructorSpec<PortableForm>,
    metadata: &InkProject,
) -> rust::Tokens {
    let conn = &new_name("conn", constructor.args());
    let salt = &new_name("salt", constructor.args());
    let data = &new_name("data", constructor.args());
    let account_id = &new_name("account_id", constructor.args());
    let code_hash_name = &new_name("code_hash", constructor.args());

    quote! {
        $(docs(constructor.docs()))
        #[allow(dead_code)]
        pub async fn $(&constructor.label)<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
            $(conn): &C,
            $(salt): Vec<u8>,
            $(message_args(&constructor.args, metadata))
        ) -> Result<Self, E> {
            let $(data) = $(gather_args(constructor.selector().to_bytes(), constructor.args()));
            let $(code_hash_name) = $(format!("{:?}", hex_to_bytes(code_hash)));
            let $(account_id) = conn.instantiate($(code_hash_name), $(salt), $(data)).await?;
            Ok(Self { account_id })
        }
        $[ '\n' ]
    }
}

/// Generates a function wrapping a contract message send.
fn define_message(message: &MessageSpec<PortableForm>, metadata: &InkProject) -> rust::Tokens {
    if message.mutates() {
        define_mutator(message, metadata)
    } else {
        define_reader(message, metadata)
    }
}

/// Generates a function wrapping a contract message that only reads from the contract.
fn define_reader(message: &MessageSpec<PortableForm>, metadata: &InkProject) -> rust::Tokens {
    let conn = &new_name("conn", message.args());
    let data = &new_name("data", message.args());

    quote! {
        $(docs(message.docs()))
        #[allow(dead_code)]
        pub async fn $(message.label())<E, C: ink_wrapper_types::Connection<E>>(
            &self,
            $(conn): &C, $(message_args(message.args(), metadata))
        ) ->
            Result<$(type_ref(message.return_type().opt_type().unwrap().ty().id(), metadata)), E>
        {
            let $(data) = $(gather_args(message.selector().to_bytes(), message.args()));
            $(conn).read(self.account_id, $(data)).await
        }

        $[ '\n' ]
    }
}

/// Generates a function wrapping a contract message that mutates the contract.
fn define_mutator(message: &MessageSpec<PortableForm>, metadata: &InkProject) -> rust::Tokens {
    let conn = &new_name("conn", message.args());
    let data = &new_name("data", message.args());

    quote! {
        $(docs(message.docs()))
        #[allow(dead_code)]
        pub async fn $(message.label())<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
            &self, $(conn): &C,
            $(message_args(message.args(), metadata))
        ) -> Result<TxInfo, E>
        {
            let $(data) = $(gather_args(message.selector().to_bytes(), message.args()));
            $(conn).exec(self.account_id, $(data)).await
        }

        $[ '\n' ]
    }
}

/// Generates a block of statesments that pack the selector and arguments into a SCALE encoded vector of bytes.
///
/// The intention is to assign the result to a variable.
fn gather_args(selector: &[u8], args: &[MessageParamSpec<PortableForm>]) -> rust::Tokens {
    let data = &new_name("data", args);

    quote! {
        $(if args.is_empty() {
            vec!$(format!("{:?}", &selector));
        } else {
            {
                let mut $(data) = vec!$(format!("{:?}", &selector));
                $(for arg in args {
                    $(arg.label()).encode_to(&mut $(data));
                })
                $(data)
            }
        })
    }
}

/// Generates a list of arguments for a constructor/message wrapper.
fn message_args(args: &[MessageParamSpec<PortableForm>], metadata: &InkProject) -> rust::Tokens {
    quote! {
        $(for arg in args {
            $(arg.label()): $(type_ref(arg.ty().ty().id(), metadata)),
        })
    }
}

/// Generates a type reference to the given type (for example to use as an argument type, return type, etc.).
fn type_ref(id: u32, metadata: &InkProject) -> String {
    let typ = resolve(metadata, id);

    match typ.type_def() {
        TypeDef::Primitive(primitive) => type_ref_primitive(primitive),
        TypeDef::Tuple(tuple) => type_ref_tuple(tuple, metadata),
        TypeDef::Composite(_) => type_ref_generic(typ, metadata),
        TypeDef::Variant(_) => type_ref_generic(typ, metadata),
        TypeDef::Array(array) => type_ref_array(array, metadata),
        TypeDef::Sequence(sequence) => type_ref_sequence(sequence, metadata),
        TypeDef::Compact(compact) => type_ref_compact(compact, metadata),
        TypeDef::BitSequence(_) => panic!("Bit sequences are not supported yet."),
    }
}

/// Generates a type reference to a (potentially generic) type by name.
fn type_ref_generic(typ: &Type<PortableForm>, metadata: &InkProject) -> String {
    let mut generics = String::new();
    let mut first = true;

    for param in typ.type_params() {
        if first {
            first = false;
        } else {
            generics.push_str(", ");
        }

        generics.push_str(&type_ref(param.ty().unwrap().id(), metadata));
    }

    format!("{}<{}>", typ.qualified_name(), generics)
}

/// Generates a type reference to a primitive type.
fn type_ref_primitive(primitive: &TypeDefPrimitive) -> String {
    match primitive {
        TypeDefPrimitive::U8 => "u8".to_string(),
        TypeDefPrimitive::I8 => "i8".to_string(),
        TypeDefPrimitive::U16 => "u16".to_string(),
        TypeDefPrimitive::I16 => "i16".to_string(),
        TypeDefPrimitive::U32 => "u32".to_string(),
        TypeDefPrimitive::I32 => "i32".to_string(),
        TypeDefPrimitive::U64 => "u64".to_string(),
        TypeDefPrimitive::I64 => "i64".to_string(),
        TypeDefPrimitive::U128 => "u128".to_string(),
        TypeDefPrimitive::I128 => "i128".to_string(),
        TypeDefPrimitive::U256 => "u256".to_string(),
        TypeDefPrimitive::I256 => "i256".to_string(),
        TypeDefPrimitive::Bool => "bool".to_string(),
        TypeDefPrimitive::Char => "char".to_string(),
        TypeDefPrimitive::Str => "String".to_string(),
    }
}

/// Generates a type reference to a tuple type.
fn type_ref_tuple(tuple: &TypeDefTuple<PortableForm>, metadata: &InkProject) -> String {
    format!(
        "({})",
        tuple
            .fields()
            .iter()
            .map(|t| type_ref(t.id(), metadata))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Generates a type reference to an array type.
fn type_ref_array(array: &TypeDefArray<PortableForm>, metadata: &InkProject) -> String {
    format!(
        "[{}; {}]",
        type_ref(array.type_param().id(), metadata),
        array.len()
    )
}

/// Generates a type reference to a sequence type.
fn type_ref_sequence(sequence: &TypeDefSequence<PortableForm>, metadata: &InkProject) -> String {
    format!("Vec<{}>", type_ref(sequence.type_param().id(), metadata))
}

/// Generates a type reference to a compact type.
fn type_ref_compact(compact: &TypeDefCompact<PortableForm>, metadata: &InkProject) -> String {
    format!(
        "scale::Compact<{}>",
        type_ref(compact.type_param().id(), metadata)
    )
}

/// Generates a docstring from a list of doc lines.
fn docs(lines: &[String]) -> String {
    lines
        .iter()
        .map(|line| format!("/// {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Resolves the type with the given ID.
///
/// Panics if the type cannot be found in the metadata. We should only use types that are mentioned in the metadata
/// file, so any type that cannot be found is a bug in the code generator or the metadata file.
fn resolve(metadata: &InkProject, id: u32) -> &Type<PortableForm> {
    metadata
        .registry()
        .resolve(id)
        .unwrap_or_else(|| panic!("Type {} not found", id))
}

/// Generates a name not already used by one of the arguments.
fn new_name(name: &str, args: &[MessageParamSpec<PortableForm>]) -> String {
    let mut name = name.to_string();

    while args.iter().any(|arg| arg.label() == &name) {
        name.push('_');
    }

    name
}

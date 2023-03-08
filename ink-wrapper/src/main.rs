use anyhow::Result;
use clap::Parser;
use genco::fmt;
use genco::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    contract: Contract,
    source: Source,
    types: Vec<Type>,
    spec: Spec,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Contract {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Spec {
    messages: Vec<Message>,
    constructors: Vec<Constructor>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Constructor {
    selector: String,
    label: String,
    args: Vec<Arg>,
    #[serde(rename = "returnType")]
    return_type: TypeRef,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    label: String,
    args: Vec<Arg>,
    #[serde(rename = "returnType")]
    return_type: TypeRef,
    mutates: bool,
    selector: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Arg {
    label: String,
    #[serde(rename = "type")]
    typ: TypeRef,
}

#[derive(Debug, Serialize, Deserialize)]
struct TypeRef {
    #[serde(rename = "type")]
    id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Type {
    id: u32,
    #[serde(rename = "type")]
    typ: TypeSpec,
}

#[derive(Debug, Serialize, Deserialize)]
struct TypeSpec {
    #[serde(default = "Vec::new")]
    path: Vec<String>,
    #[serde(default = "Vec::new")]
    params: Vec<TypeRef>,
    def: TypeDef,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum TypeDef {
    Primitive { primitive: String },
    Tuple { tuple: Vec<u32> },
    Variant { variant: VariantDef },
    Composite { composite: CompositeDef },
}

#[derive(Debug, Serialize, Deserialize)]
struct VariantDef {
    variants: Vec<Variant>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompositeDef {
    // TODO composite with unnamed fields
    // TODO empty composite
    fields: Vec<Field>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Field {
    name: Option<String>,
    #[serde(rename = "type")]
    typ: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Variant {
    name: String,
    #[serde(default = "Vec::new")]
    fields: Vec<Field>,
}

enum Fields {
    Named(Vec<(String, u32)>),
    Unnamed(Vec<u32>),
}

impl From<Vec<Field>> for Fields {
    fn from(fields: Vec<Field>) -> Self {
        if fields.iter().all(|f| f.name.is_none()) {
            Fields::Unnamed(fields.iter().map(|f| f.typ).collect())
        } else {
            Fields::Named(
                fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone().unwrap_or_else(|| {
                                panic!("{:?} has a mix of named and unnamed fields", fields)
                            }),
                            f.typ,
                        )
                    })
                    .collect(),
            )
        }
    }
}

impl Type {
    fn is_primitive(&self) -> bool {
        self.typ.def.is_primitive()
    }

    fn is_ink(&self) -> bool {
        self.typ.path.len() > 0 && self.typ.path[0] == "ink_primitives"
    }

    fn is_builtin(&self) -> bool {
        self.typ.path.len() == 1
    }

    fn qualified_name(&self) -> String {
        if self.is_ink() {
            self.typ.path.join("::")
        } else {
            self.typ.path.last().unwrap().to_string()
        }
    }
}

impl TypeDef {
    fn is_primitive(&self) -> bool {
        matches!(self, TypeDef::Primitive { .. })
    }
}

impl Variant {
    fn fields(&self) -> Fields {
        self.fields.clone().into()
    }
}

impl CompositeDef {
    fn fields(&self) -> Fields {
        self.fields.clone().into()
    }
}

impl Message {
    fn selector_bytes(&self) -> Vec<u8> {
        hex_to_bytes(&self.selector)
    }
}

impl Constructor {
    fn selector_bytes(&self) -> Vec<u8> {
        hex_to_bytes(&self.selector)
    }
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex.replace("0x", "")).unwrap()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let metadata = fs::read_to_string(args.metadata)?;
    let metadata: Metadata = serde_json::from_str(&metadata)?;

    let types = metadata
        .types
        .iter()
        .map(|t| (t.id, t))
        .collect::<HashMap<_, _>>();

    let tokens: rust::Tokens = generate(&metadata, &types);

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(4));

    let config = rust::Config::default().with_default_import(rust::ImportMode::Qualified);

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}

fn generate(metadata: &Metadata, types: &HashMap<u32, &Type>) -> rust::Tokens {
    let encode = rust::import("scale", "Encode").with_alias("_");

    quote! {
        $(register(encode))

        $(for typ in types.values() {
            $(if !typ.is_primitive() && !typ.is_ink() && !typ.is_builtin() {
                $(define_type(typ, &types))
            })
        })

        pub struct Instance {
            account_id: ink_primitives::AccountId,
        }

        impl From<ink_primitives::AccountId> for Instance {
            fn from(account_id: ink_primitives::AccountId) -> Self {
                Self { account_id }
            }
        }

        impl Instance {
            $(for constructor in metadata.spec.constructors.iter() {
                $(define_constructor(&metadata.source.hash, constructor, &types)) $['\n']
            })

            $(for message in metadata.spec.messages.iter() {
                $(define_message(message, &types))
            })
        }
    }
}

fn define_type(typ: &Type, types: &HashMap<u32, &Type>) -> rust::Tokens {
    match &typ.typ.def {
        TypeDef::Primitive { .. } => rust::Tokens::new(),
        TypeDef::Variant { variant, .. } => define_variant(typ, &variant, types),
        TypeDef::Composite { composite, .. } => define_composite(typ, &composite, types),
        TypeDef::Tuple { .. } => rust::Tokens::new(),
    }
}

fn define_variant(typ: &Type, variant: &VariantDef, types: &HashMap<u32, &Type>) -> rust::Tokens {
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
        pub enum $(typ.qualified_name()) {
            $(for variant in &variant.variants {
                $(match variant.fields() {
                    Fields::Named(fields) => {
                        $(&variant.name) {
                            $(for (name, typ) in fields {
                                $(name): $(type_ref(types[&typ], types)),
                            })
                        },
                    },
                    Fields::Unnamed(fields) => {
                        $(&variant.name) (
                            $(for typ in fields {
                                $(type_ref(types[&typ], types)),
                            })
                        ),
                    },
                })
            })
        }
    }
}

fn define_composite(
    typ: &Type,
    composite: &CompositeDef,
    types: &HashMap<u32, &Type>,
) -> rust::Tokens {
    quote! {
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
        $(match composite.fields() {
            Fields::Named(fields) => {
                pub struct $(typ.qualified_name()) {
                    $(for (name, typ) in fields {
                        pub $(name): $(type_ref(types[&typ], types)),
                    })
                }
            },

            Fields::Unnamed(fields) => {
                pub struct $(typ.qualified_name()) (
                    $(for typ in fields {
                        pub $(type_ref(types[&typ], types)),
                    })
                );
            },
        })
    }
}

fn define_constructor(
    code_hash: &str,
    constructor: &Constructor,
    types: &HashMap<u32, &Type>,
) -> rust::Tokens {
    quote! {
        #[allow(dead_code)]
        pub async fn $(&constructor.label)<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
            conn: &C,
            salt: Vec<u8>,
            $(message_args(&constructor.args, types))
        ) -> Result<Self, E> {
            $(gather_args(&constructor.selector_bytes(), &constructor.args))
            let code_hash = $(format!("{:?}", hex_to_bytes(code_hash)));
            let account_id = conn.instantiate(code_hash, salt, data).await?;
            Ok(Self { account_id })
        }
    }
}

fn define_message(message: &Message, types: &HashMap<u32, &Type>) -> rust::Tokens {
    if message.mutates {
        define_mutator(message, types)
    } else {
        define_reader(message, types)
    }
}

fn define_reader(message: &Message, types: &HashMap<u32, &Type>) -> rust::Tokens {
    quote! {
        #[allow(dead_code)]
        pub async fn $(&message.label)<E, C: ink_wrapper_types::Connection<E>>(
            &self,
            conn: &C, $(message_args(&message.args, types))
        ) ->
            Result<$(type_ref(types[&message.return_type.id], types)), E>
        {
            $(gather_args(&message.selector_bytes(), &message.args))
            conn.read(self.account_id, data).await
        }
    }
}

fn define_mutator(message: &Message, types: &HashMap<u32, &Type>) -> rust::Tokens {
    quote! {
        #[allow(dead_code)]
        pub async fn $(&message.label)<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
            &self, conn: &C,
            $(message_args(&message.args, types))
        ) -> Result<TxInfo, E>
        {
            $(gather_args(&message.selector_bytes(), &message.args))
            conn.exec(self.account_id, data).await
        }
    }
}

fn gather_args(selector: &Vec<u8>, args: &Vec<Arg>) -> rust::Tokens {
    quote! {
        $(if args.len() == 0 {
            let data = vec!$(format!("{:?}", &selector));
        } else {
            let mut data = vec!$(format!("{:?}", &selector));
            $(for arg in args {
                $(&arg.label).encode_to(&mut data);
            })
        })
    }
}

fn message_args(args: &Vec<Arg>, types: &HashMap<u32, &Type>) -> rust::Tokens {
    quote! {
        $(for arg in args {
            $(&arg.label): $(type_ref(types[&arg.typ.id], types)),
        })
    }
}

fn type_ref(typ: &Type, types: &HashMap<u32, &Type>) -> String {
    match &typ.typ.def {
        TypeDef::Primitive { primitive } => primitive.clone(),
        TypeDef::Tuple { tuple } => format!(
            "({})",
            tuple
                .iter()
                .map(|t| type_ref(types[&t], types))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => {
            let params = typ
                .typ
                .params
                .iter()
                .map(|p| type_ref(types[&p.id], types))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", typ.qualified_name(), params)
        }
    }
}

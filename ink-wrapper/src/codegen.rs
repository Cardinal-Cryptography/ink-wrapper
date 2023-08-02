use std::collections::HashMap;

use genco::prelude::*;
use ink_metadata::{ConstructorSpec, EventSpec, InkProject, MessageParamSpec, MessageSpec};
use scale_info::{
    form::PortableForm, Type, TypeDef, TypeDefArray, TypeDefCompact, TypeDefComposite,
    TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

use crate::extensions::*;

type MessageList<'a> = Vec<&'a MessageSpec<PortableForm>>;

/// Generates the full wrapper for the contract.
pub fn generate(
    metadata: &InkProject,
    code_hash: String,
    wasm_path: Option<String>,
) -> rust::Tokens {
    let encode = rust::import("scale", "Encode").with_alias("_");
    let (top_level_messages, trait_messages) = group_messages(metadata);

    quote! {
        $("// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).")

        $(register(encode))

        #[allow(dead_code)]
        pub const CODE_HASH: [u8; 32] = $(format!("{:?}", hex_to_bytes(&code_hash)));

        $(for typ in &metadata.registry().types {
            $(if typ.ty.is_custom() {
                $(define_type(&typ.ty, metadata))
            })
        })

        pub mod event {
            #[allow(dead_code, clippy::large_enum_variant)]
            #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
            pub enum Event {
                $(for event in metadata.spec().events() {
                    $(define_event(event, metadata))
                })
            }
        }


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

        impl ink_wrapper_types::EventSource for Instance {
            type Event = event::Event;
        }

        $(for (trait_name, messages) in trait_messages {
            $(define_trait(&trait_name, &messages, metadata))
        })

        $(if let Some(wasm_path) = wasm_path {
            $(define_upload(&wasm_path))
        })

        impl Instance {
            $(for constructor in metadata.spec().constructors().iter() {
                $(define_constructor(constructor, metadata)) $['\n']
            })

            $(for message in top_level_messages {
                $(define_message(message, "pub", metadata))
            })
        }
    }
}

fn define_upload(wasm_path: &str) -> rust::Tokens {
    quote! {
        #[allow(dead_code)]
        pub fn upload() -> ink_wrapper_types::UploadCall
        {
            let wasm = include_bytes!($(quoted(wasm_path)));
            ink_wrapper_types::UploadCall::new(wasm.to_vec(), CODE_HASH)
        }
    }
}

/// Define a group of messages with a common prefix (e.g. `PSP22::`).
///
/// These messages will be grouped into a trait and implemented for the contract to avoid name clashes.
fn define_trait(
    trait_name: &str,
    messages: &[&MessageSpec<PortableForm>],
    metadata: &InkProject,
) -> rust::Tokens {
    quote! {
        #[async_trait::async_trait]
        pub trait $(trait_name) {
            $(for message in messages {
                $(define_message_head(message, "", metadata));
            })
        }

        #[async_trait::async_trait]
        impl $(trait_name) for Instance {
            $(for message in messages {
                $(define_message(message, "", metadata))
            })
        }

        $[ '\n' ]
    }
}

fn define_message_head(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> rust::Tokens {
    if message.mutates() {
        define_mutator_head(message, visibility, metadata)
    } else {
        define_reader_head(message, visibility, metadata)
    }
}

/// Group messages by their "trait" prefix (for example groups all messages with a `PSP22::` prefix together).
///
/// Returns the "main" group without any prefix as a special group (first member of the result pair).
fn group_messages(metadata: &InkProject) -> (MessageList, HashMap<String, MessageList>) {
    let mut top_level_messages = Vec::new();
    let mut trait_messages = HashMap::new();

    for message in metadata.spec().messages() {
        match message.trait_name() {
            None => top_level_messages.push(message),
            Some(trait_name) => {
                trait_messages
                    .entry(trait_name.clone())
                    .or_insert_with(Vec::new)
                    .push(message);
            }
        }
    }

    (top_level_messages, trait_messages)
}

/// Generates a type definition for a custom type used in the contract.
fn define_type(typ: &Type<PortableForm>, metadata: &InkProject) -> rust::Tokens {
    match &typ.type_def {
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
            $(for variant in &variant.variants {
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
    constructor: &ConstructorSpec<PortableForm>,
    metadata: &InkProject,
) -> rust::Tokens {
    let data = &new_name("data", constructor.args());

    quote! {
        $(docs(constructor.docs()))
        #[allow(dead_code, clippy::too_many_arguments)]
        pub fn $(&constructor.label)($(message_args(&constructor.args, metadata))) ->
            $(if *constructor.payable() {
                ink_wrapper_types::InstantiateCallNeedsValue<Self>
            } else {
                ink_wrapper_types::InstantiateCall<Self>
            })
        {
            let $(data) = $(gather_args(constructor.selector().to_bytes(), constructor.args()));
            $(if *constructor.payable() {
                ink_wrapper_types::InstantiateCallNeedsValue::new(CODE_HASH, $(data))
            } else {
                ink_wrapper_types::InstantiateCall::new(CODE_HASH, $(data))
            })
        }
        $[ '\n' ]
    }
}

/// Generates a function wrapping a contract message send.
fn define_message(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> rust::Tokens {
    if message.mutates() {
        define_mutator(message, visibility, metadata)
    } else {
        define_reader(message, visibility, metadata)
    }
}

/// Generates a function wrapping a contract message that only reads from the contract.
fn define_reader(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> rust::Tokens {
    let data = &new_name("data", message.args());

    quote! {
        $(docs(message.docs()))
        #[allow(dead_code, clippy::too_many_arguments)]
        $(define_reader_head(message, visibility, metadata))
        {
            let $(data) = $(gather_args(message.selector().to_bytes(), message.args()));
            ink_wrapper_types::ReadCall::new(self.account_id, $(data))
        }

        $[ '\n' ]
    }
}

fn define_reader_head(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> rust::Tokens {
    quote! {
        $(visibility) fn $(message.method_name())(&self, $(message_args(message.args(), metadata))) ->
            ink_wrapper_types::ReadCall<$(type_ref(message.return_type().opt_type().unwrap().ty().id, metadata))>
    }
}

/// Generates a function wrapping a contract message that mutates the contract.
fn define_mutator(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> rust::Tokens {
    let data = &new_name("data", message.args());

    quote! {
        $(docs(message.docs()))
        #[allow(dead_code, clippy::too_many_arguments)]
        $(define_mutator_head(message, visibility, metadata))
        {
            let $(data) = $(gather_args(message.selector().to_bytes(), message.args()));
            $(if message.payable() {
                ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, $(data))
            } else {
                ink_wrapper_types::ExecCall::new(self.account_id, $(data))
            })
        }

        $[ '\n' ]
    }
}

fn define_mutator_head(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> rust::Tokens {
    quote! {
        $(visibility) fn $(message.method_name())(&self, $(message_args(message.args(), metadata))) ->
            $(if message.payable() {
                ink_wrapper_types::ExecCallNeedsValue
            } else {
                ink_wrapper_types::ExecCall
            })
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
            $(arg.label()): $(type_ref(arg.ty().ty().id, metadata)),
        })
    }
}

/// Generates an event definition as a variant in the `Event` enum.
///
/// Note that these definitions are hidden in a module to avoid name clashes (just in case someone uses `Event` as a
/// type name), so references to types defined in the contract need to be prefixed with `super::`.
fn define_event(event: &EventSpec<PortableForm>, metadata: &InkProject) -> rust::Tokens {
    quote! {
        $(docs(event.docs()))
        $(event.label()) {
            $(for field in event.args() {
                $(docs(field.docs()))
                $(field.label()): $(type_ref_prefix(field.ty().ty().id, metadata, "super::")),
            })
        },

        $[ '\n' ]
    }
}

/// Generates a type reference to the given type (for example to use as an argument type, return type, etc.).
fn type_ref(id: u32, metadata: &InkProject) -> String {
    type_ref_prefix(id, metadata, "")
}

/// Generates a type reference to the given type (for example to use as an argument type, return type, etc.).
///
/// The `prefix` is prepended to the type name if the type is a custom type.
fn type_ref_prefix(id: u32, metadata: &InkProject, prefix: &str) -> String {
    let typ = resolve(metadata, id);
    let generic_prefix = if typ.is_custom() { prefix } else { "" };

    match &typ.type_def {
        TypeDef::Primitive(primitive) => type_ref_primitive(primitive),
        TypeDef::Tuple(tuple) => type_ref_tuple(tuple, metadata, prefix),
        TypeDef::Composite(_) => type_ref_generic(typ, metadata, generic_prefix),
        TypeDef::Variant(_) => type_ref_generic(typ, metadata, generic_prefix),
        TypeDef::Array(array) => type_ref_array(array, metadata, prefix),
        TypeDef::Sequence(sequence) => type_ref_sequence(sequence, metadata, prefix),
        TypeDef::Compact(compact) => type_ref_compact(compact, metadata, prefix),
        TypeDef::BitSequence(_) => panic!("Bit sequences are not supported yet."),
    }
}

/// Generates a type reference to a (potentially generic) type by name.
fn type_ref_generic(typ: &Type<PortableForm>, metadata: &InkProject, prefix: &str) -> String {
    let mut generics = String::new();
    let mut first = true;

    for param in &typ.type_params {
        if first {
            first = false;
        } else {
            generics.push_str(", ");
        }

        generics.push_str(&type_ref_prefix(param.ty.unwrap().id, metadata, prefix));
    }

    format!("{}{}<{}>", prefix, typ.qualified_name(), generics)
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
fn type_ref_tuple(
    tuple: &TypeDefTuple<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> String {
    format!(
        "({})",
        tuple
            .fields
            .iter()
            .map(|t| type_ref_prefix(t.id, metadata, prefix))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Generates a type reference to an array type.
fn type_ref_array(
    array: &TypeDefArray<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> String {
    format!(
        "[{}; {}]",
        type_ref_prefix(array.type_param.id, metadata, prefix),
        array.len
    )
}

/// Generates a type reference to a sequence type.
fn type_ref_sequence(
    sequence: &TypeDefSequence<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> String {
    format!(
        "Vec<{}>",
        type_ref_prefix(sequence.type_param.id, metadata, prefix)
    )
}

/// Generates a type reference to a compact type.
fn type_ref_compact(
    compact: &TypeDefCompact<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> String {
    format!(
        "scale::Compact<{}>",
        type_ref_prefix(compact.type_param.id, metadata, prefix)
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

/// Parses a hex string ("0x1234...") into a byte vector.
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex.replace("0x", "")).unwrap()
}

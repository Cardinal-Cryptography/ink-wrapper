use std::collections::HashMap;

use ink_metadata::{ConstructorSpec, EventSpec, InkProject, MessageParamSpec, MessageSpec};
use proc_macro2::Ident;
use quote::*;
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
) -> proc_macro2::TokenStream {
    let (top_level_messages, trait_messages) = group_messages(metadata);

    let code_hash = hex_to_bytes(&code_hash);

    let upload = define_upload(wasm_path);

    let custom_types = define_custom_types(metadata);

    let events = define_events(metadata);

    let traits = define_traits(metadata, trait_messages);

    let impl_instance = define_impl_instance(metadata, top_level_messages);

    quote! {
        // This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).")

        use scale::Encode as _;

        #[allow(dead_code)]
        pub const CODE_HASH: [u8; 32] = [#(#code_hash),*];

        #(#custom_types)*

        pub mod event {
            #[allow(dead_code, clippy::large_enum_variant)]
            #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
            pub enum Event {
                #(#events),*
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

        #(#traits)*

        #upload

        #impl_instance
    }
}

fn define_custom_types(
    metadata: &InkProject,
) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    metadata
        .registry()
        .types
        .iter()
        .filter(|typ| typ.ty.is_custom())
        .map(|typ| define_type(&typ.ty, metadata))
}

fn define_events(metadata: &InkProject) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    metadata
        .spec()
        .events()
        .iter()
        .map(|event| define_event(event, metadata))
}

fn define_traits(
    metadata: &InkProject,
    trait_messages: HashMap<String, MessageList>,
) -> Vec<proc_macro2::TokenStream> {
    trait_messages
        .iter()
        .map(|(trait_name, messages)| define_trait(trait_name, messages, metadata))
        .collect()
}

fn define_impl_instance(
    metadata: &InkProject,
    top_level_messages: Vec<&MessageSpec<PortableForm>>,
) -> proc_macro2::TokenStream {
    let constructors = metadata
        .spec()
        .constructors()
        .iter()
        .map(|constructor| define_constructor(constructor, metadata));
    let messages = top_level_messages
        .iter()
        .map(|message| define_message(message, "pub", metadata));

    quote! {
        impl Instance {
            #(#constructors)*

            #(#messages)*
        }
    }
}

// If wasm_path is defined, returns a function that uploads the contract to the chain.
// If `None`, returns empty `quote!{}` - a noop.
fn define_upload(wasm_path: Option<String>) -> proc_macro2::TokenStream {
    match wasm_path {
        Some(wasm_path) => quote! {
            #[allow(dead_code)]
            pub fn upload() -> ink_wrapper_types::UploadCall
            {
                let wasm = include_bytes!(#wasm_path);
                ink_wrapper_types::UploadCall::new(wasm.to_vec(), CODE_HASH)
            }
        },
        None => quote! {},
    }
}

/// Define a group of messages with a common prefix (e.g. `PSP22::`).
///
/// These messages will be grouped into a trait and implemented for the contract to avoid name clashes.
fn define_trait<'a, 'b>(
    trait_name: &str,
    messages: &[&'b MessageSpec<PortableForm>],
    metadata: &'a InkProject,
) -> proc_macro2::TokenStream
where
    'a: 'b,
{
    let trait_name = format_ident!("{}", trait_name);
    let trait_messages = messages
        .iter()
        .map(|m| define_message_head(m, "", metadata));

    let impl_messages = messages.iter().map(|m| define_message(m, "", metadata));
    quote! {
        pub trait #trait_name {
            #(#trait_messages;)*
        }

        impl #trait_name for Instance {
            #(#impl_messages)*
        }
    }
}

fn define_message_head(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
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
fn define_type(typ: &Type<PortableForm>, metadata: &InkProject) -> proc_macro2::TokenStream {
    match &typ.type_def {
        TypeDef::Variant(variant) => define_enum(typ, variant, metadata),
        TypeDef::Composite(composite) => define_composite(typ, composite, metadata),
        _ => quote! {},
    }
}

fn named_variant(
    name: &str,
    fields: &[(String, u32)],
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|(name, typ)| {
        let typ = type_ref(*typ, metadata);
        let name = format_ident!("{}", name);
        quote! {
            #name: #typ
        }
    });
    let name = format_ident!("{}", name);
    quote! {
        #name {
            #(#fields),*
        }
    }
}

fn unnamed_variant(name: &str, fields: &[u32], metadata: &InkProject) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|typ| {
        let typ = type_ref(*typ, metadata);
        quote! {
            #typ
        }
    });
    let name = format_ident!("{}", name);
    quote! {
        #name (
            #(#fields),*
        )
    }
}

/// Generates a type definition for an enum.
fn define_enum(
    typ: &Type<PortableForm>,
    variant: &TypeDefVariant<PortableForm>,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let typ = typ.qualified_name();
    let variants = variant
        .variants
        .iter()
        .map(|variant| match variant.aggregate_fields() {
            Fields::Named(fields) => named_variant(&variant.name, &fields, metadata),
            Fields::Unnamed(fields) => unnamed_variant(&variant.name, &fields, metadata),
        });
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
        pub enum #typ {
            #(#variants),*
        }
    }
}

/// Generates a type definition for a struct.
fn define_composite(
    typ: &Type<PortableForm>,
    composite: &TypeDefComposite<PortableForm>,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    match composite.aggregate_fields() {
        Fields::Named(fields) => {
            let typ = typ.qualified_name();
            let fields = fields.iter().map(|(name, typ)| {
                let typ = type_ref(*typ, metadata);
                let name = format_ident!("{}", name);
                quote! {
                    pub #name: #typ
                }
            });
            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
                pub struct #typ {
                    #(#fields),*
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let typ = typ.qualified_name();
            let fields = unnamed.iter().map(|typ| {
                let typ = type_ref(*typ, metadata);
                quote! {
                    pub #typ
                }
            });
            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
                pub struct #typ (
                    #(#fields),*
                );
            }
        }
    }
}

/// Generates a function wrapping a contract constructor.
fn define_constructor(
    constructor: &ConstructorSpec<PortableForm>,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let data_ident = &new_name("data", constructor.args());
    let docs = quote_docs(constructor.docs());
    let label = format_ident!("{}", constructor.label());
    let args = message_args(constructor.args(), metadata);
    let ret_res = if *constructor.payable() {
        quote! { ink_wrapper_types::InstantiateCallNeedsValue<Self> }
    } else {
        quote! { ink_wrapper_types::InstantiateCall<Self> }
    };
    let data = gather_args(constructor.selector().to_bytes(), constructor.args());
    let body = if *constructor.payable() {
        quote! {
            let #data_ident = #data;
            ink_wrapper_types::InstantiateCallNeedsValue::new(CODE_HASH, #data_ident)
        }
    } else {
        quote! {
            let #data_ident = #data;
            ink_wrapper_types::InstantiateCall::new(CODE_HASH, #data_ident)
        }
    };

    quote! {
        #docs
        #[allow(dead_code, clippy::too_many_arguments)]
        pub fn #label ( #args ) -> #ret_res {
            #body
        }
    }
}

/// Generates a function wrapping a contract message send.
fn define_message(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
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
) -> proc_macro2::TokenStream {
    let data_ident = &new_name("data", message.args());
    let docs = quote_docs(message.docs());
    let reader_head = define_reader_head(message, visibility, metadata);
    let args = gather_args(message.selector().to_bytes(), message.args());

    quote! {
        #docs
        #[allow(dead_code, clippy::too_many_arguments)]
        #reader_head
        {
            let #data_ident = #args;
            ink_wrapper_types::ReadCall::new(self.account_id, #data_ident)
        }
    }
}

fn quote_visibility(visibility: &str) -> proc_macro2::TokenStream {
    if visibility.is_empty() {
        quote! {}
    } else {
        let v = format_ident!("{}", visibility);
        quote! { #v }
    }
}

fn define_reader_head(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let method = format_ident!("{}", message.method_name());
    let args = message_args(message.args(), metadata);
    let read_call_type = type_ref(message.return_type().opt_type().unwrap().ty().id, metadata);
    let ret_type = if message.payable() {
        quote! { ink_wrapper_types::ReadCallNeedsValue<#read_call_type> }
    } else {
        quote! { ink_wrapper_types::ReadCall<#read_call_type> }
    };
    let visibility = quote_visibility(visibility);
    quote! {
        #visibility fn #method (&self, #args) -> #ret_type
    }
}

/// Generates a function wrapping a contract message that mutates the contract.
fn define_mutator(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let data_ident = &new_name("data", message.args());
    let data = gather_args(message.selector().to_bytes(), message.args());
    let docs = quote_docs(message.docs());
    let mutator_head = define_mutator_head(message, visibility, metadata);
    let res = if message.payable() {
        quote! {
            ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, #data_ident)
        }
    } else {
        quote! {
            ink_wrapper_types::ExecCall::new(self.account_id, #data_ident)
        }
    };
    quote! {
        #docs
        #[allow(dead_code, clippy::too_many_arguments)]
        #mutator_head
        {
            let #data_ident = #data;
            #res
        }
    }
}

fn define_mutator_head(
    message: &MessageSpec<PortableForm>,
    visibility: &str,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let method = format_ident!("{}", message.method_name());
    let message_args = message_args(message.args(), metadata);
    let ret_type = if message.payable() {
        quote! { ink_wrapper_types::ExecCallNeedsValue }
    } else {
        quote! { ink_wrapper_types::ExecCall }
    };
    let visibility = quote_visibility(visibility);
    quote! {
        #visibility fn #method (&self, #message_args) -> #ret_type
    }
}

/// Generates a block of statements that pack the selector and arguments into a SCALE encoded vector of bytes.
///
/// The intention is to assign the result to a variable.
fn gather_args(
    selector: &[u8],
    args: &[MessageParamSpec<PortableForm>],
) -> proc_macro2::TokenStream {
    let selector_deref: Vec<u8> = selector.to_vec();
    if args.is_empty() {
        quote! {
            vec![#(#selector_deref),*]
        }
    } else {
        let data_ident = format_ident!("{}", new_name("data", args));
        let args = args.iter().map(|arg| {
            let arg_label = format_ident!("{}", arg.label());
            quote! { #arg_label.encode_to(&mut #data_ident) }
        });
        quote!({
            let mut #data_ident = vec![#(#selector_deref),*];
            #(#args;)*
            #data_ident
        })
    }
}

/// Generates a list of arguments for a constructor/message wrapper.
fn message_args(
    args: &[MessageParamSpec<PortableForm>],
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let args = args.iter().map(|arg| {
        let arg_label = format_ident!("{}", arg.label());
        let arg_type = type_ref(arg.ty().ty().id, metadata);
        quote! { #arg_label: #arg_type }
    });
    quote! { #(#args),* }
}

/// Generates an event definition as a variant in the `Event` enum.
///
/// Note that these definitions are hidden in a module to avoid name clashes (just in case someone uses `Event` as a
/// type name), so references to types defined in the contract need to be prefixed with `super::`.
fn define_event(
    event: &EventSpec<PortableForm>,
    metadata: &InkProject,
) -> proc_macro2::TokenStream {
    let event_docs = quote_docs(event.docs());
    let event_label = format_ident!("{}", event.label());
    let event_fields = event.args().iter().map(|field| {
        let field_docs = quote_docs(field.docs());
        let field_label = format_ident!("{}", field.label());
        let field_type = type_ref_prefix(field.ty().ty().id, metadata, "super");
        quote! {
           #field_docs
           #field_label: #field_type
        }
    });
    quote! {
        #event_docs
        #event_label {
            #(#event_fields),*
        }
    }
}

/// Generates a type reference to the given type (for example to use as an argument type, return type, etc.).
fn type_ref(id: u32, metadata: &InkProject) -> proc_macro2::TokenStream {
    type_ref_prefix(id, metadata, "")
}

/// Generates a type reference to the given type (for example to use as an argument type, return type, etc.).
///
/// The `prefix` is prepended to the type name if the type is a custom type.
fn type_ref_prefix(id: u32, metadata: &InkProject, prefix: &str) -> proc_macro2::TokenStream {
    let typ = resolve(metadata, id);

    match &typ.type_def {
        TypeDef::Primitive(primitive) => {
            let t = type_ref_primitive(primitive);
            quote! { #t }
        }
        TypeDef::Tuple(tuple) => type_ref_tuple(tuple, metadata, prefix),
        TypeDef::Composite(_) => type_ref_generic(typ, metadata, prefix),
        TypeDef::Variant(_) => type_ref_generic(typ, metadata, prefix),
        TypeDef::Array(array) => type_ref_array(array, metadata, prefix),
        TypeDef::Sequence(sequence) => type_ref_sequence(sequence, metadata, prefix),
        TypeDef::Compact(compact) => type_ref_compact(compact, metadata, prefix),
        TypeDef::BitSequence(_) => panic!("Bit sequences are not supported yet."),
    }
}

/// Generates a type reference to a (potentially generic) type by name.
fn type_ref_generic(
    typ: &Type<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> proc_macro2::TokenStream {
    let generics = if typ.type_params.is_empty() {
        quote! {}
    } else {
        let generics = typ.type_params.iter().map(|param| {
            let param = param.ty.unwrap();
            let param = type_ref_prefix(param.id, metadata, prefix);
            quote! { #param }
        });
        quote! { <#(#generics),*> }
    };

    let prefix = if prefix.is_empty() || !typ.is_custom() {
        quote! {}
    } else {
        let prefix_ident = format_ident!("{}", prefix);
        quote! { #prefix_ident:: }
    };

    let qualified_name = typ.qualified_name();
    quote! { #prefix #qualified_name #generics }
}

/// Generates a type reference to a primitive type.
fn type_ref_primitive(primitive: &TypeDefPrimitive) -> proc_macro2::TokenStream {
    match primitive {
        TypeDefPrimitive::U8 => quote! { u8 },
        TypeDefPrimitive::I8 => quote! { i8 },
        TypeDefPrimitive::U16 => quote! { u16 },
        TypeDefPrimitive::I16 => quote! { i16 },
        TypeDefPrimitive::U32 => quote! { u32 },
        TypeDefPrimitive::I32 => quote! { i32 },
        TypeDefPrimitive::U64 => quote! { u64 },
        TypeDefPrimitive::I64 => quote! { i64 },
        TypeDefPrimitive::I128 => quote! { i128 },
        TypeDefPrimitive::U128 => quote! { u128 },
        TypeDefPrimitive::U256 => quote! { u256 },
        TypeDefPrimitive::I256 => quote! { i256 },
        TypeDefPrimitive::Bool => quote! { bool },
        TypeDefPrimitive::Char => quote! { char },
        TypeDefPrimitive::Str => quote! { String },
    }
}

/// Generates a type reference to a tuple type.
fn type_ref_tuple(
    tuple: &TypeDefTuple<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> proc_macro2::TokenStream {
    let typs = tuple
        .fields
        .iter()
        .map(|t| type_ref_prefix(t.id, metadata, prefix));
    quote! {
        (#(#typs),*)
    }
}

/// Generates a type reference to an array type.
fn type_ref_array(
    array: &TypeDefArray<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> proc_macro2::TokenStream {
    let typ = type_ref_prefix(array.type_param.id, metadata, prefix);
    // Cast to usize as otherwise we will get compilation errors on other archs.
    let len = array.len as usize;
    quote! {
        [ #typ ; #len ]
    }
}

/// Generates a type reference to a sequence type.
fn type_ref_sequence(
    sequence: &TypeDefSequence<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> proc_macro2::TokenStream {
    let typ = type_ref_prefix(sequence.type_param.id, metadata, prefix);
    quote! {
        Vec<#typ>
    }
}

/// Generates a type reference to a compact type.
fn type_ref_compact(
    compact: &TypeDefCompact<PortableForm>,
    metadata: &InkProject,
    prefix: &str,
) -> proc_macro2::TokenStream {
    let typ = type_ref_prefix(compact.type_param.id, metadata, prefix);
    quote! {
        scale::Compact<#typ>
    }
}

fn quote_docs(lines: &[String]) -> proc_macro2::TokenStream {
    if lines.is_empty() {
        quote! {}
    } else {
        let d = format!(
            "{}",
            lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join("")
        );
        quote! { #[doc = #d] }
    }
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
fn new_name(name: &str, args: &[MessageParamSpec<PortableForm>]) -> Ident {
    let mut name = name.to_string();

    while args.iter().any(|arg| arg.label() == &name) {
        name.push('_');
    }

    format_ident!("{}", name)
}

/// Parses a hex string ("0x1234...") into a byte vector.
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex.replace("0x", "")).unwrap()
}

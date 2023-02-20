use anyhow::Result;
use clap::Parser;
use codegen::Scope;
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
    types: Vec<Type>,
    spec: Spec,
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
    fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Field {
    name: String,
    #[serde(rename = "type")]
    typ: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Variant {
    name: String,
    #[serde(default = "Vec::new")]
    fields: Vec<VariantField>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VariantField {
    name: Option<String>,
    #[serde(rename = "type")]
    typ: u32,
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

impl Message {
    fn selector_bytes(&self) -> Vec<u8> {
        let selector = self.selector.replace("0x", "");
        hex::decode(selector).unwrap()
    }
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

    let mut result = Scope::new();
    result.import("scale", "Encode");

    let c = result.new_struct("Instance");
    c.vis("pub");
    c.new_field("account_id", "ink_primitives::AccountId");

    let i = result.new_impl("From<ink_primitives::AccountId> for Instance");
    i.new_fn("from")
        .arg("account_id", "ink_primitives::AccountId")
        .ret("Self")
        .line("Self { account_id }");

    for typ in types.values() {
        if !typ.is_primitive() && !typ.is_ink() && !typ.is_builtin() {
            define_type(&mut result, typ, &types);
        }
    }

    let i = result.new_impl("Instance");

    for constructor in metadata.spec.constructors {
        let f = i
            .new_fn(&constructor.label)
            .set_async(true)
            .vis("pub")
            .generic("E")
            .generic("TxInfo")
            .generic("C: ink_wrapper_types::SignedConnection<TxInfo, E>")
            .arg("conn", "&C")
            .ret(type_ref(types[&constructor.return_type.id], &types));

        for arg in constructor.args {
            f.arg(&arg.label, type_ref(types[&arg.typ.id], &types));
        }

        f.line("Ok(())");
    }

    for message in metadata.spec.messages {
        // TODO args named by us are unhygienic
        let f = i
            .new_fn(&message.label)
            .set_async(true)
            .vis("pub")
            .arg_ref_self()
            .arg("conn", "&C");

        if message.args.len() > 0 {
            f.line(format!(
                "let mut args = vec!{:?};",
                message.selector_bytes()
            ));
        } else {
            f.line(format!("let args = vec!{:?};", message.selector_bytes()));
        }

        if message.mutates {
            f.generic("TxInfo")
                .generic("E")
                .generic("C: ink_wrapper_types::SignedConnection<TxInfo, E>")
                .ret("Result<TxInfo, E>");
        } else {
            f.generic("E")
                .generic("C: ink_wrapper_types::Connection<E>")
                .ret(format!(
                    "Result<{}, E>",
                    type_ref(types[&message.return_type.id], &types)
                ));
        };

        for arg in message.args {
            f.arg(&arg.label, type_ref(types[&arg.typ.id], &types));
            f.line(format!("{}.encode_to(&mut args);", arg.label));
        }

        if message.mutates {
            f.line("conn.exec(self.account_id, args).await");
        } else {
            f.line("conn.read(self.account_id, args).await");
        }
    }

    println!("{}", result.to_string());

    Ok(())
}

fn define_type(scope: &mut Scope, typ: &Type, types: &HashMap<u32, &Type>) {
    match &typ.typ.def {
        TypeDef::Primitive { .. } => (),
        TypeDef::Variant { variant, .. } => define_variant(scope, typ, &variant, types),
        TypeDef::Composite { composite, .. } => define_composite(scope, typ, &composite, types),
        TypeDef::Tuple { .. } => (),
    }
}

fn define_variant(
    scope: &mut Scope,
    typ: &Type,
    variant: &VariantDef,
    types: &HashMap<u32, &Type>,
) {
    let definition = scope.new_enum(&typ.qualified_name());
    definition.vis("pub");
    definition.derive("Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode");

    for variant in &variant.variants {
        let variant_def = definition.new_variant(&variant.name);

        for field in variant.fields.iter() {
            match &field.name {
                Some(name) => variant_def.named(&name, &type_ref(types[&field.typ], types)),
                None => variant_def.tuple(&type_ref(types[&field.typ], types)),
            };
        }
    }
}

fn define_composite(
    scope: &mut Scope,
    typ: &Type,
    composite: &CompositeDef,
    types: &HashMap<u32, &Type>,
) {
    let definition = scope.new_struct(&typ.qualified_name());
    definition.vis("pub");
    definition.derive("Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode");

    for field in &composite.fields {
        let field_type = type_ref(types[&field.typ], types);
        definition.new_field(&field.name, &field_type);
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

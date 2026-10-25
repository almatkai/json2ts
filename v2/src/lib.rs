//! Offline reproduction of the command's json-ts 1.6.4 inference pipeline.
mod names;
mod parser;
use names::{interface_name, needs_quotes, pascal};
use parser::{Kind, Node};
use std::collections::HashSet;

pub const MAX_INPUT_BYTES: usize = 1_048_576;
pub const MAX_NAME_BYTES: usize = 16_384;
pub const MAX_OUTPUT_BYTES: usize = 4_194_304;
pub const MAX_NODES: usize = 20_000;
pub const MAX_DEPTH: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Any,
    Number,
    String,
    Boolean,
    Null,
    Reference(String),
    Literal(Vec<Member>),
    Array(Option<Box<Type>>),
    Parenthesized(Box<Type>),
    Union(Vec<Type>),
}
impl Type {
    fn kind(&self) -> u8 {
        match self {
            Self::Any => 0,
            Self::Number => 1,
            Self::String => 2,
            Self::Boolean => 3,
            Self::Null => 4,
            Self::Reference(_) => 5,
            Self::Literal(_) => 6,
            Self::Array(_) => 7,
            Self::Parenthesized(_) => 8,
            Self::Union(_) => 9,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    name: String,
    ty: Type,
    optional: bool,
}
#[derive(Clone)]
struct Interface {
    name: String,
    members: Vec<Member>,
}

fn property(name: &str) -> Result<String, String> {
    names::property(name)
}
fn array_name(name: &str) -> Result<String, String> {
    Ok(format!(
        "{}{}",
        if needs_quotes(name)? { "_" } else { "" },
        pascal(&format!("{name}_Item"))?
    ))
}
fn primitive(kind: Kind) -> Type {
    match kind {
        Kind::Null => Type::Null,
        Kind::True | Kind::False => Type::Boolean,
        Kind::String => Type::String,
        Kind::Number => Type::Number,
        _ => Type::Any,
    }
}
fn array_type(node: &Node) -> Result<Type, String> {
    let mut kinds = Vec::new();
    for n in &node.body {
        if !kinds.contains(&n.kind) {
            kinds.push(n.kind);
        }
    }
    let element = match kinds.as_slice() {
        [] => Type::Any,
        [Kind::Object] => Type::Reference(array_name(&node.name)?),
        [k] => primitive(*k),
        [Kind::True, Kind::False] | [Kind::False, Kind::True] => Type::Boolean,
        _ if kinds
            .iter()
            .all(|k| !matches!(k, Kind::Object | Kind::Array)) =>
        {
            Type::Parenthesized(Box::new(Type::Union(
                kinds.into_iter().map(primitive).collect(),
            )))
        }
        _ => Type::Any,
    };
    Ok(Type::Array(Some(Box::new(element))))
}
fn members(nodes: &[Node]) -> Result<Vec<Member>, String> {
    nodes
        .iter()
        .map(|n| {
            let ty = match n.kind {
                Kind::Object if n.candidate => Type::Reference(interface_name(&n.name)?),
                Kind::Object => Type::Literal(members(&n.body)?),
                Kind::Array => array_type(n)?,
                k => primitive(k),
            };
            Ok(Member {
                name: property(&n.name)?,
                ty,
                optional: false,
            })
        })
        .collect()
}
fn interfaces(nodes: &mut [Node], out: &mut Vec<Interface>) -> Result<(), String> {
    for n in nodes {
        match n.kind {
            Kind::Object => {
                // json-ts creates even inlined objects before deciding whether to retain them.
                let current = Interface {
                    name: interface_name(&n.name)?,
                    members: members(&n.body)?,
                };
                if n.candidate {
                    if out.len() >= 2048 {
                        return Err("Input exceeds interface limit (2048)".into());
                    }
                    out.push(current);
                }
                interfaces(&mut n.body, out)?;
            }
            Kind::Array => {
                let name = pascal(&format!("{}_Item", n.name))?;
                for child in &mut n.body {
                    child.name.clone_from(&name);
                }
                interfaces(&mut n.body, out)?;
            }
            _ => {}
        }
    }
    Ok(())
}
fn empty_array(t: &Type) -> bool {
    matches!(t, Type::Array(Some(e)) if **e == Type::Any)
}
fn matches(a: &Type, b: &Type) -> bool {
    a.kind() == b.kind()
        && match (a, b) {
            (Type::Array(a), Type::Array(b)) => {
                a.as_ref().map(|x| x.kind()) == b.as_ref().map(|x| x.kind())
            }
            _ => true,
        }
}
fn collapse(mut input: Vec<Interface>) -> Result<Vec<Interface>, String> {
    // The original uses a plain JS object for this map: preserve its prototype collision error.
    if input.iter().any(|i| {
        !i.members.is_empty()
            && [
                "__proto__",
                "constructor",
                "toString",
                "valueOf",
                "hasOwnProperty",
                "isPrototypeOf",
                "propertyIsEnumerable",
                "toLocaleString",
                "__defineGetter__",
                "__defineSetter__",
                "__lookupGetter__",
                "__lookupSetter__",
            ]
            .contains(&i.name.as_str())
    }) {
        return Err("Invalid JSON".into());
    }
    // Deliberately excludes the first interface's members from names, matching 1.6.4.
    for idx in 0..input.len() {
        let occurrences: Vec<_> = input
            .iter()
            .enumerate()
            .filter(|(_, i)| i.name == input[idx].name)
            .map(|(j, _)| j)
            .collect();
        if occurrences.len() > 1 {
            let names: HashSet<_> = occurrences[1..]
                .iter()
                .flat_map(|&j| input[j].members.iter().map(|m| m.name.clone()))
                .collect();
            for m in &mut input[idx].members {
                if !names.contains(&m.name) {
                    m.optional = true;
                }
            }
        }
    }
    let mut output: Vec<Interface> = Vec::new();
    for current in input {
        let Some(prev) = output.iter_mut().find(|i| i.name == current.name) else {
            output.push(current);
            continue;
        };
        let cur_names: HashSet<_> = current.members.iter().map(|m| m.name.as_str()).collect();
        let prev_names: HashSet<_> = prev.members.iter().map(|m| m.name.as_str()).collect();
        if cur_names.len() < prev_names.len() {
            for m in &mut prev.members {
                if !cur_names.contains(m.name.as_str()) {
                    m.optional = true;
                }
            }
        }
        for mut m in current.members {
            let Some(p) = prev.members.iter_mut().find(|p| p.name == m.name) else {
                m.optional = true;
                prev.members.push(m);
                continue;
            };
            if matches(&p.ty, &m.ty) {
                continue;
            }
            if let Type::Union(types) = &mut p.ty {
                if !types.iter().any(|t| t.kind() == m.ty.kind()) {
                    types.push(m.ty);
                }
            } else {
                let ty = if empty_array(&p.ty) && !empty_array(&m.ty) {
                    Type::Array(match m.ty {
                        Type::Array(e) => e,
                        _ => None,
                    })
                } else {
                    if empty_array(&m.ty) && matches!(p.ty, Type::Array(_)) {
                        continue;
                    }
                    Type::Union(vec![p.ty.clone(), m.ty])
                };
                // namedProp called again on the ALREADY quoted name, with no optional flag.
                p.name = property(&p.name)?;
                p.optional = false;
                p.ty = ty;
            }
        }
    }
    Ok(output)
}
struct Printer {
    text: String,
}
impl Printer {
    fn add(&mut self, s: &str) -> Result<(), String> {
        if self.text.len().saturating_add(s.len()) > MAX_OUTPUT_BYTES {
            return Err("Generated output exceeds 4 MiB limit".into());
        }
        self.text.push_str(s);
        Ok(())
    }
    fn indent(&mut self, depth: usize) -> Result<(), String> {
        self.add(&"    ".repeat(depth))
    }
    fn ty(&mut self, t: &Type, depth: usize) -> Result<(), String> {
        match t {
            Type::Any => self.add("any"),
            Type::Number => self.add("number"),
            Type::String => self.add("string"),
            Type::Boolean => self.add("boolean"),
            Type::Null => self.add("null"),
            Type::Reference(s) => self.add(s),
            Type::Literal(m) if m.is_empty() => self.add("{}"),
            Type::Literal(m) => self.block(m, depth),
            Type::Array(element) => {
                if let Some(e) = element {
                    self.ty(e, depth)?;
                }
                self.add("[]")
            }
            Type::Parenthesized(t) => {
                self.add("(")?;
                self.ty(t, depth)?;
                self.add(")")
            }
            Type::Union(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.add(" | ")?;
                    }
                    self.ty(t, depth)?;
                }
                Ok(())
            }
        }
    }
    fn block(&mut self, members: &[Member], depth: usize) -> Result<(), String> {
        self.add("{\n")?;
        for m in members {
            self.indent(depth + 1)?;
            self.add(&m.name)?;
            if m.optional {
                self.add("?")?;
            }
            self.add(": ")?;
            self.ty(&m.ty, depth + 1)?;
            self.add(";\n")?;
        }
        self.indent(depth)?;
        self.add("}")
    }
}
/// Exact command options: rootName from form, flow:false, prefix:"".
/// Errors do not include private input. Limits fail explicitly and never truncate.
pub fn convert(input: &str, root_name: &str) -> Result<String, String> {
    if input.is_empty() {
        return Err("Empty JSON".into());
    }
    if root_name.is_empty() {
        return Err("Empty interface name".into());
    }
    if input.len() > MAX_INPUT_BYTES {
        return Err("JSON exceeds 1 MiB limit".into());
    }
    if root_name.len() > MAX_NAME_BYTES {
        return Err("Interface name exceeds 16 KiB limit".into());
    }
    let encoded_root_name = names::units(root_name);
    let root_name = encoded_root_name.as_str();
    let mut root = parser::parse(input)?;
    let mut p = Printer {
        text: String::new(),
    };
    match root.kind {
        Kind::Object | Kind::Array => {
            let is_array = root.kind == Kind::Array;
            let body = if is_array {
                root.name = root_name.into();
                vec![root]
            } else {
                root.body
            };
            let mut wrapper = vec![Node {
                name: root_name.into(),
                kind: Kind::Object,
                candidate: true,
                body,
            }];
            let mut all = Vec::new();
            interfaces(&mut wrapper, &mut all)?;
            let all = collapse(collapse(all)?)?;
            for (idx, i) in all.iter().enumerate() {
                if is_array && idx == 0 {
                    p.add("type ")?;
                    p.add(root_name)?;
                    p.add(" = ")?;
                    p.ty(&i.members.first().ok_or("Invalid JSON")?.ty, 0)?;
                    p.add(";\n")?;
                } else {
                    p.add("interface ")?;
                    p.add(&i.name)?;
                    p.add(" ")?;
                    p.block(&i.members, 0)?;
                    p.add("\n")?;
                }
            }
        }
        k => {
            p.add("type ")?;
            p.add(root_name)?;
            p.add(" = ")?;
            p.ty(&primitive(k), 0)?;
            p.add(";")?;
        }
    }
    names::decode_units(&p.text)
}

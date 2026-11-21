// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Canonical type expressions and compiled slot defaults.

use std::collections::{BTreeMap, BTreeSet};

use capnp::{
    ErrorKind, any_pointer, any_pointer_list,
    message::{Builder, HeapAllocator},
    primitive_list,
    schema_capnp::{brand, type_, value},
    struct_list,
};

use super::{
    DescriptorError as Error, TypeId,
    descriptor::{Kind, Nodes, layout},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Atom {
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Text,
    Data,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Pointer {
    Any,
    Struct,
    List,
    Capability,
    Parameter(TypeId, u16),
    MethodParameter(u16),
}

#[derive(Clone, Copy)]
pub(super) enum TypeScope {
    Lexical(TypeId),
    MethodBrand {
        owner: TypeId,
        arity: u32,
    },
    Inline {
        owner: TypeId,
        wrapper: TypeId,
        arity: u32,
    },
}

impl TypeScope {
    pub(super) const fn owner(self) -> TypeId {
        match self {
            Self::Lexical(owner) | Self::MethodBrand { owner, .. } | Self::Inline { owner, .. } => {
                owner
            }
        }
    }
}

pub(super) type Brand = BTreeMap<TypeId, Arguments>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Type {
    Atom(Atom),
    List(Box<Self>),
    Named(Kind, TypeId, Brand),
    Pointer(Pointer),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Arguments {
    Inherited(u32),
    Bound(Vec<Type>),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum DefaultValue {
    Bits(u64),
    Bytes(Vec<u8>),
    Pointer(Option<Vec<u8>>),
}

impl Type {
    /// The units in a slot offset and its storage section, if it has storage.
    pub(super) const fn storage(&self) -> Option<(bool, u32)> {
        let bits = match self {
            Self::Atom(Atom::Void) => return None,
            Self::Atom(Atom::Bool) => 1,
            Self::Atom(Atom::Int8 | Atom::Uint8) => 8,
            Self::Atom(Atom::Int16 | Atom::Uint16) | Self::Named(Kind::Enum, ..) => 16,
            Self::Atom(Atom::Int32 | Atom::Uint32 | Atom::Float32) => 32,
            Self::Atom(Atom::Int64 | Atom::Uint64 | Atom::Float64) => 64,
            _ => return Some((true, 1)),
        };
        Some((false, bits))
    }

    pub(super) fn read(
        reader: type_::Reader<'_>,
        nodes: &Nodes<'_>,
        scope: TypeScope,
        dependencies: &mut BTreeSet<TypeId>,
    ) -> Result<Self, Error> {
        use type_::Which;
        let atom = match reader.which()? {
            Which::Void(()) => Atom::Void,
            Which::Bool(()) => Atom::Bool,
            Which::Int8(()) => Atom::Int8,
            Which::Int16(()) => Atom::Int16,
            Which::Int32(()) => Atom::Int32,
            Which::Int64(()) => Atom::Int64,
            Which::Uint8(()) => Atom::Uint8,
            Which::Uint16(()) => Atom::Uint16,
            Which::Uint32(()) => Atom::Uint32,
            Which::Uint64(()) => Atom::Uint64,
            Which::Float32(()) => Atom::Float32,
            Which::Float64(()) => Atom::Float64,
            Which::Text(()) => Atom::Text,
            Which::Data(()) => Atom::Data,
            Which::List(list) => {
                if !list.has_element_type() {
                    return Err(layout("list type has no element type"));
                }
                return Self::read(list.get_element_type()?, nodes, scope, dependencies)
                    .map(|item| Self::List(Box::new(item)));
            }
            Which::Enum(data) => {
                return Self::named(
                    Kind::Enum,
                    TypeId(data.get_type_id()),
                    data.get_brand()?,
                    nodes,
                    scope,
                    dependencies,
                );
            }
            Which::Struct(data) => {
                return Self::named(
                    Kind::Struct,
                    TypeId(data.get_type_id()),
                    data.get_brand()?,
                    nodes,
                    scope,
                    dependencies,
                );
            }
            Which::Interface(data) => {
                return Self::named(
                    Kind::Interface,
                    TypeId(data.get_type_id()),
                    data.get_brand()?,
                    nodes,
                    scope,
                    dependencies,
                );
            }
            Which::AnyPointer(pointer) => {
                return read_pointer(pointer, nodes, scope).map(Self::Pointer);
            }
        };
        Ok(Self::Atom(atom))
    }

    fn named(
        kind: Kind,
        id: TypeId,
        brand: brand::Reader<'_>,
        nodes: &Nodes<'_>,
        scope: TypeScope,
        dependencies: &mut BTreeSet<TypeId>,
    ) -> Result<Self, Error> {
        nodes.require_kind(id, kind)?;
        if kind != Kind::Interface {
            dependencies.insert(id);
        }
        Ok(Self::Named(
            kind,
            id,
            read_brand(brand, nodes, (scope, id), dependencies)?,
        ))
    }
}

fn read_pointer(
    reader: type_::any_pointer::Reader<'_>,
    nodes: &Nodes<'_>,
    context: TypeScope,
) -> Result<Pointer, Error> {
    use type_::any_pointer::{Which, unconstrained};
    Ok(match reader.which()? {
        Which::Unconstrained(value) => match value.which()? {
            unconstrained::AnyKind(()) => Pointer::Any,
            unconstrained::Struct(()) => Pointer::Struct,
            unconstrained::List(()) => Pointer::List,
            unconstrained::Capability(()) => Pointer::Capability,
        },
        Which::Parameter(value) => {
            let scope = TypeId(value.get_scope_id());
            let index = value.get_parameter_index();
            if let TypeScope::Inline { wrapper, arity, .. } = context
                && scope == wrapper
            {
                if u32::from(index) >= arity {
                    return Err(layout(format!(
                        "invalid method parameter {index} in wrapper {wrapper}"
                    )));
                }
                return Ok(Pointer::MethodParameter(index));
            }
            if !nodes.contains_scope(context.owner(), scope)?
                || u32::from(index) >= nodes.parameters(scope)?
            {
                return Err(layout(format!(
                    "invalid parameter {index} in scope {scope}"
                )));
            }
            Pointer::Parameter(scope, index)
        }
        Which::ImplicitMethodParameter(parameter) => {
            let TypeScope::MethodBrand { arity, .. } = context else {
                return Err(layout(
                    "implicit method parameter requires an RPC method brand",
                ));
            };
            let index = parameter.get_parameter_index();
            if u32::from(index) >= arity {
                return Err(layout(format!("invalid implicit method parameter {index}")));
            }
            Pointer::MethodParameter(index)
        }
    })
}

pub(super) fn read_brand(
    reader: brand::Reader<'_>,
    nodes: &Nodes<'_>,
    (context, target): (TypeScope, TypeId),
    dependencies: &mut BTreeSet<TypeId>,
) -> Result<Brand, Error> {
    let mut brand = BTreeMap::new();
    for scope in reader.get_scopes()? {
        let id = TypeId(scope.get_scope_id());
        if !nodes.contains_scope(target, id)? || brand.contains_key(&id) {
            return Err(layout(format!(
                "invalid or repeated brand scope {id} of {target}"
            )));
        }
        let count = nodes.parameters(id)?;
        if count == 0 && matches!(context, TypeScope::MethodBrand { .. }) {
            return Err(layout(format!(
                "brand scope {id} of {target} declares no parameters"
            )));
        }
        let arguments = match scope.which()? {
            brand::scope::Inherit(()) => {
                if !nodes.contains_scope(context.owner(), id)? {
                    return Err(layout(format!("unavailable inherited brand scope {id}")));
                }
                Arguments::Inherited(count)
            }
            brand::scope::Bind(bindings) => {
                let bindings = bindings?;
                if bindings.len() > count {
                    return Err(layout(format!("too many brand bindings for scope {id}")));
                }
                let mut values = Vec::new();
                for binding in bindings {
                    let value = match binding.which()? {
                        brand::binding::Unbound(()) => Type::Pointer(Pointer::Any),
                        brand::binding::Type(value) => {
                            Type::read(value?, nodes, context, dependencies)?
                        }
                    };
                    if !value.storage().is_some_and(|(pointer, _)| pointer) {
                        return Err(layout(format!("non-pointer brand binding for scope {id}")));
                    }
                    values.push(value);
                }
                if usize::try_from(count).ok() == Some(values.len())
                    && values.iter().enumerate().all(|(index, ty)| {
                        matches!(ty, Type::Pointer(Pointer::Parameter(scope, parameter))
                            if *scope == id && usize::from(*parameter) == index)
                    })
                {
                    Arguments::Inherited(count)
                } else {
                    while values.last() == Some(&Type::Pointer(Pointer::Any)) {
                        values.pop();
                    }
                    Arguments::Bound(values)
                }
            }
        };
        brand.insert(id, (count, arguments));
    }
    Ok(brand
        .into_iter()
        .filter_map(|(id, (count, arguments))| {
            (count != 0 && !matches!(&arguments, Arguments::Bound(values) if values.is_empty()))
                .then_some((id, arguments))
        })
        .collect())
}

pub(super) fn default(
    ty: &Type,
    reader: value::Reader<'_>,
    nodes: &Nodes<'_>,
) -> Result<DefaultValue, Error> {
    use value::Which;
    let bits = match (ty, reader.which()?) {
        (Type::Atom(Atom::Void), Which::Void(()))
        | (Type::Named(Kind::Interface, ..), Which::Interface(())) => 0,
        (Type::Atom(Atom::Bool), Which::Bool(value)) => u64::from(value),
        (Type::Atom(Atom::Int8), Which::Int8(value)) => u64::from(value.cast_unsigned()),
        (Type::Atom(Atom::Int16), Which::Int16(value)) => u64::from(value.cast_unsigned()),
        (Type::Atom(Atom::Int32), Which::Int32(value)) => u64::from(value.cast_unsigned()),
        (Type::Atom(Atom::Int64), Which::Int64(value)) => value.cast_unsigned(),
        (Type::Atom(Atom::Uint8), Which::Uint8(value)) => u64::from(value),
        (Type::Atom(Atom::Uint16), Which::Uint16(value)) => u64::from(value),
        (Type::Atom(Atom::Uint32), Which::Uint32(value)) => u64::from(value),
        (Type::Atom(Atom::Uint64), Which::Uint64(value)) => value,
        (Type::Atom(Atom::Float32), Which::Float32(value)) => u64::from(value.to_bits()),
        (Type::Atom(Atom::Float64), Which::Float64(value)) => value.to_bits(),
        (Type::Named(Kind::Enum, id, _), Which::Enum(value)) => {
            if u32::from(value) >= nodes.enumerant_count(*id)? {
                return Err(layout(format!(
                    "default enum ordinal {value} does not exist in {id}"
                )));
            }
            u64::from(value)
        }
        (Type::Atom(Atom::Text), Which::Text(value)) => {
            let value = value?
                .to_str()
                .map_err(|error| layout(format!("invalid UTF-8 Text default: {error}")))?;
            return Ok(DefaultValue::Bytes(value.as_bytes().to_vec()));
        }
        (Type::Atom(Atom::Data), Which::Data(value)) => {
            return Ok(DefaultValue::Bytes(value?.to_vec()));
        }
        (Type::List(element), Which::List(value)) => {
            check_list_default(value, element, nodes)?;
            return pointer_default(value);
        }
        (Type::Named(Kind::Struct, ..), Which::Struct(value)) => {
            check_struct_pointer(value)?;
            return pointer_default(value);
        }
        (Type::Pointer(kind), Which::AnyPointer(value)) => {
            check_any_pointer(value, *kind)?;
            return pointer_default(value);
        }
        _ => return Err(layout("slot type and default value do not agree")),
    };
    Ok(DefaultValue::Bits(bits))
}

fn check_any_pointer(value: any_pointer::Reader<'_>, kind: Pointer) -> Result<(), Error> {
    match kind {
        Pointer::Struct => check_struct_pointer(value)?,
        Pointer::List => {
            // The Void view checks the list kind and every element layout
            // except bit packing. Only the native bit-list refusal licenses
            // the Boolean view. Traversal and other input failures propagate.
            match value.get_as::<primitive_list::Reader<'_, ()>>() {
                Ok(_) => {}
                Err(error) if error.kind == ErrorKind::FoundBitListWhereStructListWasExpected => {
                    value.get_as::<primitive_list::Reader<'_, bool>>()?;
                }
                Err(error) => return Err(error.into()),
            }
        }
        Pointer::Capability if !value.is_null() => {
            return Err(layout("non-null capability default"));
        }
        Pointer::Any
        | Pointer::Capability
        | Pointer::Parameter(..)
        | Pointer::MethodParameter(_) => {}
    }
    Ok(())
}

fn check_list_default(
    list: any_pointer::Reader<'_>,
    element: &Type,
    nodes: &Nodes<'_>,
) -> Result<(), Error> {
    match element {
        Type::Atom(Atom::Void) => {
            list.get_as::<primitive_list::Reader<'_, ()>>()?;
        }
        Type::Atom(Atom::Bool) => {
            list.get_as::<primitive_list::Reader<'_, bool>>()?;
        }
        Type::Atom(Atom::Int8 | Atom::Uint8) => {
            list.get_as::<primitive_list::Reader<'_, u8>>()?;
        }
        Type::Atom(Atom::Int16 | Atom::Uint16) => {
            list.get_as::<primitive_list::Reader<'_, u16>>()?;
        }
        Type::Atom(Atom::Int32 | Atom::Uint32 | Atom::Float32) => {
            list.get_as::<primitive_list::Reader<'_, u32>>()?;
        }
        Type::Atom(Atom::Int64 | Atom::Uint64 | Atom::Float64) => {
            list.get_as::<primitive_list::Reader<'_, u64>>()?;
        }
        Type::Named(Kind::Enum, id, _) => {
            let count = nodes.enumerant_count(*id)?;
            for ordinal in list.get_as::<primitive_list::Reader<'_, u16>>()? {
                if u32::from(ordinal) >= count {
                    return Err(layout(format!(
                        "list default enum ordinal {ordinal} does not exist in {id}"
                    )));
                }
            }
        }
        Type::Named(Kind::Struct, ..) => {
            // Native list admission includes compatible struct encodings. The
            // descriptor supplies no generated reader for the member fields.
            list.get_as::<struct_list::Reader<'_, value::Owned>>()?;
        }
        Type::Atom(Atom::Text | Atom::Data)
        | Type::Named(Kind::Interface, ..)
        | Type::List(_)
        | Type::Pointer(_) => {
            for pointer in list.get_as::<any_pointer_list::Reader<'_>>()? {
                check_indirect_default(pointer?, element, nodes)?;
            }
        }
    }
    Ok(())
}

fn check_indirect_default(
    pointer: any_pointer::Reader<'_>,
    element: &Type,
    nodes: &Nodes<'_>,
) -> Result<(), Error> {
    match element {
        Type::List(element) => check_list_default(pointer, element, nodes),
        Type::Atom(Atom::Text) => {
            pointer
                .get_as::<capnp::text::Reader<'_>>()?
                .to_str()
                .map_err(|error| layout(format!("invalid UTF-8 Text default: {error}")))?;
            Ok(())
        }
        Type::Atom(Atom::Data) => {
            pointer.get_as::<capnp::data::Reader<'_>>()?;
            Ok(())
        }
        Type::Pointer(kind) => check_any_pointer(pointer, *kind),
        Type::Named(Kind::Interface, ..) => check_any_pointer(pointer, Pointer::Capability),
        _ => Err(layout("expected a list of pointer values")),
    }
}

fn check_struct_pointer(pointer: any_pointer::Reader<'_>) -> Result<(), Error> {
    // The native struct accessor checks pointer kind before any field is read.
    // The field layout of this concrete reader is unused.
    pointer.get_as::<value::Reader<'_>>()?;
    Ok(())
}

fn pointer_default(pointer: any_pointer::Reader<'_>) -> Result<DefaultValue, Error> {
    if pointer.is_null() {
        return Ok(DefaultValue::Pointer(None));
    }
    let size = pointer.target_size()?;
    if size.cap_count != 0 || size.word_count >= super::descriptor::MAX_WORDS as u64 {
        return Err(layout(
            "pointer default exceeds the admitted capability-free word bound",
        ));
    }
    let words = u32::try_from(size.word_count + 1)
        .map_err(|_| layout("pointer default size exceeds the canonical segment bound"))?;
    let mut canonical = Builder::new(HeapAllocator::new().first_segment_words(words));
    canonical.set_root_canonical(pointer)?;
    Ok(DefaultValue::Pointer(Some(
        canonical.get_segments_for_output()[0].to_vec(),
    )))
}

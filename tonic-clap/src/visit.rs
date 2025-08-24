#![allow(dead_code)]

use bevy_reflect::TypeInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TCFieldType {
    String,
    U8,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Vec(Box<TCFieldType>),
    Option(Box<TCFieldType>),
    Struct {
        // prefix: Vec<String>,
        name: String,
        fields: Vec<TCStructField>,
    },
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TCStructField {
    pub prefix: Vec<String>, // field path from the root struct to this field.
    pub field_name: String,
    pub field_type: TCFieldType,
}

fn parse_type_path(type_info: &TypeInfo, prefix: Vec<String>) -> TCFieldType {
    let type_path = strip_namespace(type_info.type_path());
    match type_path {
        "String" => TCFieldType::String,
        "i32" => TCFieldType::I32,
        "i64" => TCFieldType::I64,
        "f32" => TCFieldType::F32,
        "f64" => TCFieldType::F64,
        "bool" => TCFieldType::Bool,
        "u8" => TCFieldType::U8,
        _ if type_path.starts_with("Vec<") && type_path.ends_with(">") => {
            let inner = &type_path[4..type_path.len() - 1];
            let list_info = if let bevy_reflect::TypeInfo::List(list_info) = type_info {
                list_info
            } else {
                panic!("not an Vec: {type_info:?}");
            };

            let inner_type_info = list_info.item_info().unwrap_or_else(|| {
                panic!("Could not get item info for list: {list_info:?}");
            });
            assert_eq!(
                strip_namespace(inner),
                strip_namespace(inner_type_info.type_path())
            );
            let inner_type = parse_type_path(inner_type_info, prefix.clone());
            TCFieldType::Vec(Box::new(inner_type))
        }
        _ if type_path.starts_with("Option<") && type_path.ends_with(">") => {
            let inner = &type_path[7..type_path.len() - 1];

            let enum_info = if let bevy_reflect::TypeInfo::Enum(enum_info) = type_info {
                enum_info
            } else {
                panic!("not an option.");
            };
            assert!(enum_info.variant("Some").is_some() && enum_info.variant("None").is_some());

            // Get the Some variant which contains the inner type
            let some_variant = enum_info
                .variant("Some")
                .unwrap_or_else(|| panic!("Could not find Some variant for: {enum_info:?}"));
            // Option's Some variant is a tuple variant with one field containing T
            let tuple_variant = some_variant.as_tuple_variant().unwrap_or_else(|_| {
                panic!("Some variant is not a tuple variant for: {enum_info:?}",)
            });
            let inner_field = tuple_variant.field_at(0).unwrap_or_else(|| {
                panic!("Could not access inner field at index 0 for: {enum_info:?}",)
            });
            // Recursively call with the inner type's info
            let inner_type_info = inner_field.type_info().unwrap_or_else(|| {
                panic!("Could not get type info for inner field: {inner_field:?}",)
            });
            assert_eq!(
                strip_namespace(inner),
                strip_namespace(inner_type_info.type_path())
            );
            let inner_type = parse_type_path(inner_type_info, prefix.clone());
            TCFieldType::Option(Box::new(inner_type))
        }
        _ => {
            // map is not supported yet
            if matches!(
                type_info,
                bevy_reflect::TypeInfo::Map(_) | bevy_reflect::TypeInfo::Opaque(_)
            ) {
                return TCFieldType::Unknown(type_info.type_path().to_string());
            }
            // assume it is a struct.
            parse_struct(type_info, prefix)
        }
    }
}

// ignore namespace. If it is nested type only strips the outermost layer
fn strip_namespace(type_path: &str) -> &str {
    // find the first <
    let end = type_path.find('<').unwrap_or(type_path.len());

    // find the first :: before <
    let first_colon = type_path[..end].rfind("::");
    let index = match first_colon {
        Some(pos) => std::cmp::min(pos + 2, type_path.len()),
        None => 0,
    };
    &type_path[index..]
}

fn parse_struct(type_info: &TypeInfo, prefix: Vec<String>) -> TCFieldType {
    let struct_info = if let TypeInfo::Struct(info) = type_info {
        info
    } else {
        panic!("expect a struct at {prefix:?} : {type_info:?}");
    };

    let prefix_outer = prefix.clone();
    let fields = struct_info
        .iter()
        .map(|field| {
            let field_name = field.name();
            let mut prefix_inner = prefix_outer.clone();
            prefix_inner.push(field_name.to_string());
            let field_type = parse_type_path(field.type_info().unwrap(), prefix_inner.clone());
            TCStructField {
                prefix: prefix_outer.clone(), // struct fields them selfs should not contain its own field.
                field_name: field_name.to_string(),
                field_type,
            }
        })
        .collect();

    let struct_name = strip_namespace(struct_info.type_path());

    TCFieldType::Struct {
        name: struct_name.to_string(),
        fields,
    }
}

impl TCFieldType {
    pub fn parse(type_info: &TypeInfo) -> Self {
        parse_struct(type_info, vec![])
    }

    /// Visit all nested fields
    /// Callback is only applied on primitive types.
    pub fn visit_nested(&self, f: &mut dyn FnMut(&Vec<String>, &String, &TCFieldType)) {
        match &self {
            TCFieldType::Struct { fields, .. } => {
                for field in fields {
                    if field.field_type.is_primitive() {
                        f(&field.prefix, &field.field_name, &field.field_type);
                    } else {
                        field.field_type.visit_nested(f);
                    }
                }
            }
            TCFieldType::Option(inner) => {
                inner.visit_nested(f);
            }
            TCFieldType::Unknown(_) => {
                // skip well known unknown types.
            }
            _ => {
                panic!("cannot visit on unsupported type: {self:?}")
            }
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(
            &self,
            TCFieldType::I32
                | TCFieldType::I64
                | TCFieldType::F32
                | TCFieldType::F64
                | TCFieldType::Bool
                | TCFieldType::U8
                | TCFieldType::String
                | TCFieldType::Vec(_)
        )
    }

    pub fn get_clap_value_parse(&self) -> (clap::builder::ValueParser, clap::ArgAction) {
        assert!(self.is_primitive());
        match &self {
            TCFieldType::U8 => (clap::value_parser!(u8).into(), clap::ArgAction::Set),
            TCFieldType::I32 => (clap::value_parser!(i32).into(), clap::ArgAction::Set),
            TCFieldType::I64 => (clap::value_parser!(i64).into(), clap::ArgAction::Set),
            TCFieldType::F32 => (clap::value_parser!(f32).into(), clap::ArgAction::Set),
            TCFieldType::F64 => (clap::value_parser!(f64).into(), clap::ArgAction::Set),
            TCFieldType::Bool => (clap::value_parser!(bool), clap::ArgAction::Set),
            TCFieldType::String => (clap::value_parser!(String), clap::ArgAction::Set),
            TCFieldType::Vec(inner) => {
                assert!(inner.is_primitive());
                (inner.get_clap_value_parse().0, clap::ArgAction::Append)
            }
            _ => panic!("not a primitive type"),
        }
    }

    pub fn display_primitive_type(&self) -> &str {
        match &self {
            TCFieldType::I32 => "i32",
            TCFieldType::I64 => "i64",
            TCFieldType::F32 => "f32",
            TCFieldType::F64 => "f64",
            TCFieldType::Bool => "bool",
            TCFieldType::String => "String",
            TCFieldType::U8 => "u8",
            TCFieldType::Vec(inner) => {
                assert!(
                    inner.is_primitive(),
                    "Vec elements must be primitive: {inner:?} :{self:?}"
                );
                let s = "Vec<".to_string() + inner.display_primitive_type() + ">";
                Box::leak(s.into_boxed_str())
            }
            _ => panic!("not a primitive type: {self:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_reflect::Typed;

    use super::*;

    #[test]
    fn test_strip_namespace() {
        assert_eq!(strip_namespace("std::string::String"), "String");
        assert_eq!(strip_namespace("std::vec::Vec"), "Vec");
        assert_eq!(strip_namespace("Vec"), "Vec");
        assert_eq!(
            strip_namespace("alloc::vec::Vec<alloc::string::String>"),
            "Vec<alloc::string::String>"
        );
    }

    #[derive(bevy_reflect::Reflect, Default)]
    struct Struct1 {
        field0: i32,
        field1: String,
        field2: Struct2,
        field3: Option<i64>,
    }

    #[derive(bevy_reflect::Reflect, Default)]
    struct Struct2 {
        field0: f64,
        field1: Vec<String>,
        field2: Struct3,
    }

    #[derive(bevy_reflect::Reflect, Default)]
    struct Struct3 {
        field0: i32,
    }

    #[test]
    fn test_parse_type() {
        let parsed = TCFieldType::parse(Struct1::type_info());
        if let TCFieldType::Struct { name, fields } = parsed {
            assert_eq!(name, "Struct1");
            assert_eq!(fields.len(), 4);
            assert_eq!(
                fields[0],
                TCStructField {
                    prefix: vec![],
                    field_name: "field0".into(),
                    field_type: TCFieldType::I32
                }
            );
            assert_eq!(
                fields[1],
                TCStructField {
                    prefix: vec![],
                    field_name: "field1".into(),
                    field_type: TCFieldType::String
                }
            );
            assert_eq!(
                fields[2],
                TCStructField {
                    prefix: vec![],
                    field_name: "field2".into(),
                    field_type: TCFieldType::Struct {
                        name: "Struct2".into(),
                        fields: vec![
                            TCStructField {
                                prefix: vec!["field2".to_string()],
                                field_name: "field0".into(),
                                field_type: TCFieldType::F64
                            },
                            TCStructField {
                                prefix: vec!["field2".to_string()],
                                field_name: "field1".into(),
                                field_type: TCFieldType::Vec(Box::new(TCFieldType::String))
                            },
                            TCStructField {
                                prefix: vec!["field2".to_string()],
                                field_name: "field2".into(),
                                field_type: TCFieldType::Struct {
                                    name: "Struct3".into(),
                                    fields: vec![TCStructField {
                                        prefix: vec!["field2".to_string(), "field2".to_string()],
                                        field_name: "field0".into(),
                                        field_type: TCFieldType::I32
                                    },]
                                }
                            },
                        ]
                    }
                }
            );
            assert_eq!(
                fields[3],
                TCStructField {
                    prefix: vec![],
                    field_name: "field3".into(),
                    field_type: TCFieldType::Option(Box::new(TCFieldType::I64))
                }
            );
        } else {
            panic!("not a struct");
        };
    }
}

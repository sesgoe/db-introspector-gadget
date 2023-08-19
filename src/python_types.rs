use std::collections::HashMap;

use convert_case::{Case, Casing};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum PythonDataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Date,
    Binary,
    Any,
}

impl PythonDataType {
    pub(crate) fn as_primitive_type_str(&self) -> String {
        match self {
            PythonDataType::String => "str",
            PythonDataType::Integer => "int",
            PythonDataType::Float => "float",
            PythonDataType::Boolean => "bool",
            PythonDataType::DateTime => "datetime.datetime",
            PythonDataType::Date => "datetime.date",
            PythonDataType::Binary => "bytes",
            PythonDataType::Any => "Any",
        }
        .to_string()
    }
}

impl From<String> for PythonDataType {
    // some patterns are also defined for mysql, which makes the postgres patterns unreachable
    // this is fine, and the lint is disabled mostly for better readability
    #[allow(unreachable_patterns)]
    fn from(data_type: String) -> Self {
        match data_type.as_str() {
            // mysql
            "varchar" | "longtext" | "text" | "json" | "char" | "mediumtext" | "enum" | "set" => {
                PythonDataType::String
            }
            "int" | "bigint" | "smallint" => PythonDataType::Integer,
            "float" | "double" | "decimal" => PythonDataType::Float,
            "tinyint" => PythonDataType::Boolean,
            "datetime" | "timestamp" => PythonDataType::DateTime,
            "date" => PythonDataType::Date,
            "binary" | "blob" | "mediumblob" | "longblob" | "varbinary" => PythonDataType::Binary,

            // postgres
            "integer" | "bigint" => PythonDataType::Integer,
            "boolean" => PythonDataType::Boolean,
            "character varying" | "jsonb" | "USER-DEFINED" | "text" => PythonDataType::String, // user-defined are typically enums
            "date" => PythonDataType::Date,
            "double precision" | "numeric" => PythonDataType::Float,
            "timestamp with time zone" => PythonDataType::DateTime,

            _ => PythonDataType::Any,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct PythonDictProperty {
    pub(crate) name: String,
    pub(crate) nullable: bool,
    pub(crate) data_type: PythonDataType,
}

impl PythonDictProperty {
    pub(crate) fn as_type_str(&self) -> String {
        if self.nullable {
            format!("{} | None", self.data_type.as_primitive_type_str())
        } else {
            self.data_type.as_primitive_type_str()
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) struct PythonDict {
    pub(crate) name: String,
    pub(crate) properties: Vec<PythonDictProperty>,
}

impl PythonDict {
    pub(crate) fn to_typed_dict_class_string(&self) -> String {
        let mut type_string = format!("class {}(TypedDict):\n", self.name);
        for property in &self.properties {
            type_string.push_str(&format!(
                "    {}: {}\n",
                property.name,
                property.as_type_str()
            ));
        }
        type_string
    }

    pub(crate) fn to_backwards_compat_typed_dict_class_string(&self) -> String {
        let mut type_string = format!("{} = TypedDict('{}', {{\n", self.name, self.name);
        for property in &self.properties {
            type_string.push_str(&format!(
                "    '{}': {},\n",
                property.name,
                property.as_type_str()
            ));
        }
        type_string.push_str("})");
        type_string
    }
}

pub(crate) fn write_table_dict_hashmap_to_string(
    tables: HashMap<String, Vec<PythonDictProperty>>,
) -> String {
    let mut result = String::from("import datetime\nfrom typing import TypedDict, Any\n\n\n");

    let python_dicts = tables
        .iter()
        .map(|(name, properties)| PythonDict {
            name: name.to_case(Case::Pascal),
            properties: properties.to_vec(),
        })
        .filter(|dict| !dict.name.contains('$')) // prevents weirdness with some system tables
        .sorted_by_key(|f| f.name.clone())
        .map(|dict| {
            let mut iter = dict.properties.iter();

            let starts_with_number =
                |p: &PythonDictProperty| p.name.chars().next().unwrap().is_numeric();
            let contains_space = |p: &PythonDictProperty| p.name.contains(' ');
            let contains_keyword = |p: &PythonDictProperty| p.name == "from";

            let requires_backwards_compat =
                iter.any(|p| starts_with_number(p) || contains_space(p) || contains_keyword(p));

            if requires_backwards_compat {
                dict.to_backwards_compat_typed_dict_class_string()
            } else {
                dict.to_typed_dict_class_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    result.push_str(python_dicts.as_str());

    result
}

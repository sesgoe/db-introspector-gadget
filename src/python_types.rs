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
    pub(crate) fn as_property_type_str(&self) -> String {
        if self.nullable {
            format!("{} | None", self.data_type.as_primitive_type_str())
        } else {
            self.data_type.as_primitive_type_str()
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) struct PythonTypedDict {
    pub(crate) name: String,
    pub(crate) properties: Vec<PythonDictProperty>,
}

impl PythonTypedDict {
    pub(crate) fn as_typed_dict_class_string(&self) -> String {
        let mut type_string = format!("class {}(TypedDict):\n", self.name);
        for property in &self.properties {
            type_string.push_str(&format!(
                "    {}: {}\n",
                property.name,
                property.as_property_type_str()
            ));
        }
        type_string
    }

    pub(crate) fn as_backwards_compat_typed_dict_class_string(&self) -> String {
        let mut type_string = format!("{} = TypedDict('{}', {{\n", self.name, self.name);
        for property in &self.properties {
            type_string.push_str(&format!(
                "    '{}': {},\n",
                property.name,
                property.as_property_type_str()
            ));
        }
        type_string.push_str("})");
        type_string
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum PythonDataType {
    String,
    Integer,
    Long,
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
            PythonDataType::Long => "long",
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
    fn from(data_type: String) -> Self {
        match data_type.as_str() {
            //both
            "text" => PythonDataType::String,
            "date" => PythonDataType::Date,
            "bigint" => PythonDataType::Long,

            // mysql
            "varchar" | "longtext" | "json" | "char" | "mediumtext" | "enum" | "set" => {
                PythonDataType::String
            }
            "int" | "smallint" => PythonDataType::Integer,
            "float" | "double" | "decimal" => PythonDataType::Float,
            "tinyint" => PythonDataType::Boolean,
            "datetime" | "timestamp" => PythonDataType::DateTime,
            "binary" | "blob" | "mediumblob" | "longblob" | "varbinary" => PythonDataType::Binary,

            // postgres
            "integer" => PythonDataType::Integer,
            "boolean" => PythonDataType::Boolean,
            "character varying" | "jsonb" | "USER-DEFINED" => PythonDataType::String, // user-defined are typically enums for type-inference purposes
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
    pub(crate) fn as_typed_dict_class_str(&self) -> String {
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

    pub(crate) fn as_backwards_compat_typed_dict_class_str(&self) -> String {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_non_nullable_property_type_str() {
        let pdp = PythonDictProperty {
            name: String::from("some_property"),
            nullable: false,
            data_type: PythonDataType::String,
        };

        assert_eq!(pdp.as_property_type_str(), String::from("str"))
    }

    #[test]
    fn test_nullable_property_type_str() {
        let pdp = PythonDictProperty {
            name: String::from("some_property"),
            nullable: true,
            data_type: PythonDataType::String,
        };

        assert_eq!(pdp.as_property_type_str(), String::from("str | None"))
    }

    #[test]
    fn test_typed_dict_class_str() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![PythonDictProperty {
                name: String::from("some_property"),
                nullable: false,
                data_type: PythonDataType::String,
            }],
        };

        let expected = String::from("class TestTable(TypedDict):\n    some_property: str\n");

        assert_eq!(dict.as_typed_dict_class_str(), expected);
    }

    #[test]
    fn test_typed_dict_class_str_with_mult_properties() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![
                PythonDictProperty {
                    name: String::from("some_property"),
                    nullable: false,
                    data_type: PythonDataType::String,
                },
                PythonDictProperty {
                    name: String::from("some_other_property"),
                    nullable: false,
                    data_type: PythonDataType::Boolean,
                },
            ],
        };

        let expected = String::from(
            "class TestTable(TypedDict):\n    some_property: str\n    some_other_property: bool\n",
        );

        assert_eq!(dict.as_typed_dict_class_str(), expected);
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_property() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![PythonDictProperty {
                name: String::from("some_property"),
                nullable: true,
                data_type: PythonDataType::String,
            }],
        };

        let expected = String::from("class TestTable(TypedDict):\n    some_property: str | None\n");

        assert_eq!(dict.as_typed_dict_class_str(), expected);
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_and_nonnull_property() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![
                PythonDictProperty {
                    name: String::from("some_property"),
                    nullable: true,
                    data_type: PythonDataType::String,
                },
                PythonDictProperty {
                    name: String::from("some_other_property"),
                    nullable: false,
                    data_type: PythonDataType::String,
                },
            ],
        };

        let expected = String::from("class TestTable(TypedDict):\n    some_property: str | None\n    some_other_property: str\n");

        assert_eq!(dict.as_typed_dict_class_str(), expected);
    }

    #[test]
    fn test_backwards_compat_typed_dict() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![PythonDictProperty {
                name: String::from("some_property"),
                nullable: false,
                data_type: PythonDataType::String,
            }],
        };

        let expected =
            String::from("TestTable = TypedDict('TestTable', {\n    'some_property': str,\n})");

        assert_eq!(dict.as_backwards_compat_typed_dict_class_str(), expected);
    }

    #[test]
    fn test_backwards_compat_typed_dict_with_mult_and_nullable() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![
                PythonDictProperty {
                    name: String::from("some_property"),
                    nullable: false,
                    data_type: PythonDataType::String,
                },
                PythonDictProperty {
                    name: String::from("some_other_property"),
                    nullable: true,
                    data_type: PythonDataType::String,
                },
            ],
        };

        let expected =
            String::from("TestTable = TypedDict('TestTable', {\n    'some_property': str,\n    'some_other_property': str | None,\n})");

        assert_eq!(dict.as_backwards_compat_typed_dict_class_str(), expected);
    }
}

use itertools::{Itertools, Position};

use crate::MinimumPythonVersion;

/// This enum represents all the Python types we can output
/// `Any` is included as a catch-all to handle unknown database types.
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
    /// Convert a `PythonDataType` into its source code type representation
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

/// This is the primary way we convert the database INFORMATION_SCHEMA.TABLE_COLUMNS `data_type` string column
/// into given Python data types
impl From<String> for PythonDataType {
    fn from(data_type: String) -> Self {
        match data_type.as_str() {
            //both
            "text" => PythonDataType::String,
            "date" => PythonDataType::Date,
            "bigint" => PythonDataType::Integer,

            // mysql
            "varchar" | "longtext" | "json" | "char" | "mediumtext" | "enum" | "set"
            | "tinytext" => PythonDataType::String,
            "int" | "smallint" | "mediumint" => PythonDataType::Integer,
            "float" | "double" | "decimal" => PythonDataType::Float,
            "tinyint" => PythonDataType::Boolean,
            "datetime" | "timestamp" => PythonDataType::DateTime,
            "binary" | "blob" | "mediumblob" | "longblob" | "varbinary" => PythonDataType::Binary,

            // postgres
            "integer" => PythonDataType::Integer,
            "boolean" => PythonDataType::Boolean,
            "character varying" | "character" | "jsonb" | "USER-DEFINED" => PythonDataType::String, // user-defined are typically enums for type-inference purposes
            "double precision" | "numeric" => PythonDataType::Float,
            "timestamp with time zone" | "timestamp without time zone" => PythonDataType::DateTime,

            _ => PythonDataType::Any,
        }
    }
}

/// Represents a Python `TypedDict` property
/// ```text
/// class SomeTypedDict(TypedDict):
///     some_other_property: str | None
///     ^                    ^   ^
///     |                    |   |
///     name                 |   nullable
///                          data_type
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct PythonDictProperty {
    pub(crate) name: String,
    pub(crate) nullable: bool,
    pub(crate) data_type: PythonDataType,
}

impl PythonDictProperty {
    /// Builds a string representing the type of the given `PythonDictProperty`
    pub(crate) fn as_property_type_str(
        &self,
        minimum_python_version: MinimumPythonVersion,
    ) -> String {
        if self.nullable {
            match minimum_python_version {
                MinimumPythonVersion::Python3_10 => {
                    format!("{} | None", self.data_type.as_primitive_type_str())
                }
                _ => format!("Optional[{}]", self.data_type.as_primitive_type_str()),
            }
        } else {
            self.data_type.as_primitive_type_str()
        }
    }
}

/// This enum represents whether or not backward-compatible `TypedDict`
/// syntax is enabled.
///
/// It gets enabled for a single `PythonTypedDict` at a time
/// if the Python dictionary has properties that can't be represented with
/// valid Python syntax. Current examples of this include:
///
/// - If a property name starts with a numeric character
/// - If a property name contains a space
/// - If a property name is equal to a Python keyword like 'from'
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum ForcedBackwardCompat {
    Enabled,
    Disabled,
}

impl From<bool> for ForcedBackwardCompat {
    fn from(value: bool) -> Self {
        if value {
            ForcedBackwardCompat::Enabled
        } else {
            ForcedBackwardCompat::Disabled
        }
    }
}

/// Represents a full `TypedDict` definition in Python
/// ```text
/// class SomeDictionary(TypedDict):
///       ^
///       |
///       name
///     some_property: str | None
///     some_other_property: str
///     ...
///     ^
///     |
///     properties
/// ```
#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) struct PythonTypedDict {
    pub(crate) name: String,
    pub(crate) properties: Vec<PythonDictProperty>,
}

impl PythonTypedDict {
    /// Outputs a Python source string representation of this `TypedDict`
    pub(crate) fn as_typed_dict_class_str(
        &self,
        minimum_python_version: MinimumPythonVersion,
        forced_backward_compat: ForcedBackwardCompat,
    ) -> String {
        let use_alternate_syntax = minimum_python_version == MinimumPythonVersion::Python3_6
            || forced_backward_compat == ForcedBackwardCompat::Enabled;

        let mut result = if use_alternate_syntax {
            format!("{} = TypedDict('{}', {{\n", self.name, self.name)
        } else {
            format!("class {}(TypedDict):\n", self.name)
        };

        let middle_lines = self
            .properties
            .iter()
            .with_position()
            .map(
                |(position, property)| match (use_alternate_syntax, position) {
                    (true, Position::Last) | (true, Position::Only) => format!(
                        "    '{}': {}", // final property doesn't need a trailing comma
                        property.name,
                        property.as_property_type_str(minimum_python_version)
                    ),
                    (true, _) => format!(
                        "    '{}': {},", // first/middle properties need a trailing comma with this syntax
                        property.name,
                        property.as_property_type_str(minimum_python_version)
                    ),
                    (false, _) => format!(
                        "    {}: {}",
                        property.name,
                        property.as_property_type_str(minimum_python_version)
                    ),
                },
            )
            .collect::<Vec<String>>()
            .join("\n");

        result.push_str(middle_lines.as_str());
        result.push('\n');

        if use_alternate_syntax {
            result.push_str("})\n");
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    fn get_str_some_property(nullable: bool) -> PythonDictProperty {
        PythonDictProperty {
            name: String::from("some_property"),
            nullable,
            data_type: PythonDataType::String,
        }
    }

    #[test]
    fn test_non_nullable_property_type_str_python_3_6() {
        let pdp = get_str_some_property(false);

        assert_eq!(
            pdp.as_property_type_str(MinimumPythonVersion::Python3_6),
            String::from("str")
        );
    }

    #[test]
    fn test_non_nullable_property_type_str_python_3_8() {
        let pdp = get_str_some_property(false);

        assert_eq!(
            pdp.as_property_type_str(MinimumPythonVersion::Python3_8),
            String::from("str")
        );
    }

    #[test]
    fn test_non_nullable_property_type_str_python_3_10() {
        let pdp = get_str_some_property(false);

        assert_eq!(
            pdp.as_property_type_str(MinimumPythonVersion::Python3_10),
            String::from("str")
        );
    }

    #[test]
    fn test_nullable_property_type_str_python_3_6() {
        let pdp = get_str_some_property(true);

        assert_eq!(
            pdp.as_property_type_str(MinimumPythonVersion::Python3_6),
            String::from("Optional[str]")
        );
    }

    #[test]
    fn test_nullable_property_type_str_python_3_8() {
        let pdp = get_str_some_property(true);

        assert_eq!(
            pdp.as_property_type_str(MinimumPythonVersion::Python3_8),
            String::from("Optional[str]")
        );
    }

    #[test]
    fn test_nullable_property_type_str_python_3_10() {
        let pdp = get_str_some_property(true);

        assert_eq!(
            pdp.as_property_type_str(MinimumPythonVersion::Python3_10),
            String::from("str | None")
        );
    }

    #[test]
    fn test_typed_dict_class_str_python_3_6() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![get_str_some_property(false)],
        };

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str
                })
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_python_3_8() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![get_str_some_property(false)],
        };

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: str
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_python_3_10() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![get_str_some_property(false)],
        };

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: str
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_mult_properties_python_3_6() {
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

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str,
                    'some_other_property': bool
                })
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str,
                    'some_other_property': bool
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_mult_properties_python_3_8() {
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

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: str
                    some_other_property: bool
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str,
                    'some_other_property': bool
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_mult_properties_python_3_10() {
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

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: str
                    some_other_property: bool
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str,
                    'some_other_property': bool
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_property_python_3_6() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![get_str_some_property(true)],
        };

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': Optional[str]
                })
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': Optional[str]
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_property_python_3_8() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![get_str_some_property(true)],
        };

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: Optional[str]
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': Optional[str]
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_property_python_3_10() {
        let dict = PythonTypedDict {
            name: String::from("TestTable"),
            properties: vec![get_str_some_property(true)],
        };

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: str | None
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str | None
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_and_nonnull_property_python_3_6() {
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

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': Optional[str],
                    'some_other_property': str
                })
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_6,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': Optional[str],
                    'some_other_property': str
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_and_nonnull_property_python_3_8() {
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

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: Optional[str]
                    some_other_property: str
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_8,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': Optional[str],
                    'some_other_property': str
                })
            "}
        );
    }

    #[test]
    fn test_typed_dict_class_str_with_nullable_and_nonnull_property_python_3_10() {
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

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Disabled
            ),
            indoc! {"
                class TestTable(TypedDict):
                    some_property: str | None
                    some_other_property: str
            "}
        );

        assert_eq!(
            dict.as_typed_dict_class_str(
                MinimumPythonVersion::Python3_10,
                ForcedBackwardCompat::Enabled
            ),
            indoc! {"
                TestTable = TypedDict('TestTable', {
                    'some_property': str | None,
                    'some_other_property': str
                })
            "}
        );
    }
}

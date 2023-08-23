use std::collections::HashMap;

use convert_case::{Case, Casing};
use itertools::Itertools;

use crate::{
    db_introspector::TableColumnDefinition,
    python_types::{PythonDictProperty, PythonTypedDict},
    MinimumPythonVersion,
};

/// Converts a `Vec<TableColumnDefinition>` that comes from the database introspection query
/// into the `Vec<PythonTypedDict>` that is easy to manipulate into a Python source file
pub(crate) fn convert_table_column_definitions_to_python_dicts(
    table_column_definitions: Vec<TableColumnDefinition>,
) -> Vec<PythonTypedDict> {
    let mut tables_map = HashMap::<String, PythonTypedDict>::new();
    for table_column_definition in table_column_definitions {
        let dict = tables_map
            .entry(table_column_definition.table_name.clone())
            .or_insert(PythonTypedDict {
                name: table_column_definition.table_name.to_case(Case::Pascal),
                properties: vec![],
            });

        dict.properties.push(PythonDictProperty {
            name: table_column_definition.column_name,
            nullable: table_column_definition.nullable,
            data_type: table_column_definition.data_type.into(),
        });
    }

    tables_map
        .into_values()
        .sorted_by_key(|d| d.name.clone())
        .collect()
}

/// Writes the `Vec<PythonTypedDict>` into a Python source string that can then later be written to a file inside `main()`
pub(crate) fn write_python_dicts_to_str(
    dicts: Vec<PythonTypedDict>,
    minimum_python_version: MinimumPythonVersion,
) -> String {
    let mut result = match minimum_python_version {
        MinimumPythonVersion::Python3_10 => {
            String::from("import datetime\nfrom typing import Any, TypedDict\n\n\n")
            // don't need to include Optional in 3.10
        }
        _ => String::from("import datetime\nfrom typing import Any, Optional, TypedDict\n\n\n"),
    };

    let python_dicts_str = dicts
        .iter()
        .filter(|dict| !dict.name.contains('$')) // prevents weirdness with some system tables
        .filter(|dict| !dict.name.chars().next().unwrap().is_numeric())
        .sorted_by_key(|f| f.name.clone())
        .map(|dict| {
            let mut iter = dict.properties.iter();

            let starts_with_number =
                |p: &PythonDictProperty| p.name.chars().next().unwrap().is_numeric();
            let contains_space = |p: &PythonDictProperty| p.name.contains(' ');
            let is_python_keyword = |p: &PythonDictProperty| p.name == "from";

            let requires_backward_compat =
                iter.any(|p| starts_with_number(p) || contains_space(p) || is_python_keyword(p));

            dict.as_typed_dict_class_str(minimum_python_version, requires_backward_compat.into())
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    result.push_str(python_dicts_str.as_str());

    result
}

#[cfg(test)]
mod test {
    use crate::python_types::PythonDataType;
    use indoc::indoc;

    use super::*;

    #[test]
    fn convert_definitios_to_single_dict_for_single_table() {
        let table_column_definitions = vec![
            TableColumnDefinition {
                table_name: String::from("some_table"),
                column_name: String::from("column_one"),
                nullable: false,
                data_type: String::from("varchar"),
            },
            TableColumnDefinition {
                table_name: String::from("some_table"),
                column_name: String::from("column_two"),
                nullable: true,
                data_type: String::from("varchar"),
            },
        ];

        let result = convert_table_column_definitions_to_python_dicts(table_column_definitions);

        let expected = vec![PythonTypedDict {
            name: String::from("SomeTable"),
            properties: vec![
                PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                },
                PythonDictProperty {
                    name: String::from("column_two"),
                    nullable: true,
                    data_type: PythonDataType::String,
                },
            ],
        }];

        assert_eq!(result, expected)
    }

    #[test]
    fn convert_definitions_to_two_dicts_for_two_tables() {
        let table_column_definitions = vec![
            TableColumnDefinition {
                table_name: String::from("some_other_table"),
                column_name: String::from("column_one"),
                nullable: true,
                data_type: String::from("varchar"),
            },
            TableColumnDefinition {
                table_name: String::from("some_table"),
                column_name: String::from("column_one"),
                nullable: false,
                data_type: String::from("varchar"),
            },
        ];

        let result = convert_table_column_definitions_to_python_dicts(table_column_definitions);

        let expected = vec![
            PythonTypedDict {
                name: String::from("SomeOtherTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("SomeTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        assert_eq!(result, expected)
    }

    #[test]
    fn sorts_python_dict_vec_correctly() {
        let table_column_definitions = vec![
            TableColumnDefinition {
                table_name: String::from("b_table"),
                column_name: String::from("column_one"),
                nullable: false,
                data_type: String::from("varchar"),
            },
            TableColumnDefinition {
                table_name: String::from("a_table"),
                column_name: String::from("column_one"),
                nullable: true,
                data_type: String::from("varchar"),
            },
        ];

        let result = convert_table_column_definitions_to_python_dicts(table_column_definitions);

        let expected = vec![
            PythonTypedDict {
                name: String::from("ATable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("BTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        assert_eq!(result, expected)
    }

    #[test]
    fn writes_single_dict_to_string() {
        let dict = vec![PythonTypedDict {
            name: String::from("SomeTable"),
            properties: vec![
                PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                },
                PythonDictProperty {
                    name: String::from("column_two"),
                    nullable: true,
                    data_type: PythonDataType::String,
                },
            ],
        }];

        let result = write_python_dicts_to_str(dict, MinimumPythonVersion::Python3_10);
        let expected = indoc! {"
            import datetime
            from typing import Any, TypedDict


            class SomeTable(TypedDict):
                column_one: str
                column_two: str | None
            "};

        assert_eq!(result, expected)
    }

    #[test]
    fn writes_multiple_dict_to_string() {
        let dicts = vec![
            PythonTypedDict {
                name: String::from("ATable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("BTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        let result = write_python_dicts_to_str(dicts, MinimumPythonVersion::Python3_10);
        let expected = indoc! {"
            import datetime
            from typing import Any, TypedDict


            class ATable(TypedDict):
                column_one: str | None


            class BTable(TypedDict):
                column_one: str
        "};

        assert_eq!(result, expected)
    }

    #[test]
    fn ignores_writing_dict_with_dollar_sign_in_name() {
        let dicts = vec![
            PythonTypedDict {
                name: String::from("ATable$"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("BTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        let result = write_python_dicts_to_str(dicts, MinimumPythonVersion::Python3_10);
        let expected = indoc! {"
            import datetime
            from typing import Any, TypedDict


            class BTable(TypedDict):
                column_one: str
            "};

        assert_eq!(result, expected)
    }

    #[test]
    fn ignores_writing_dict_starts_with_number() {
        let dicts = vec![
            PythonTypedDict {
                name: String::from("1Table"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("BTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        let result = write_python_dicts_to_str(dicts, MinimumPythonVersion::Python3_10);
        let expected = indoc! {"
            import datetime
            from typing import Any, TypedDict


            class BTable(TypedDict):
                column_one: str
        "};

        assert_eq!(result, expected)
    }

    #[test]
    fn first_dict_backwards_compat() {
        let dicts = vec![
            PythonTypedDict {
                name: String::from("ATable"),
                properties: vec![PythonDictProperty {
                    name: String::from("1column"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("BTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        let result = write_python_dicts_to_str(dicts, MinimumPythonVersion::Python3_10);

        let expected = indoc! {"
            import datetime
            from typing import Any, TypedDict


            ATable = TypedDict('ATable', {
                '1column': str | None
            })

            class BTable(TypedDict):
                column_one: str
            "};

        assert_eq!(result, expected)
    }

    #[test]
    fn supports_python_3_6() {
        let dicts = vec![
            PythonTypedDict {
                name: String::from("ATable"),
                properties: vec![PythonDictProperty {
                    name: String::from("1column"),
                    nullable: true,
                    data_type: PythonDataType::String,
                }],
            },
            PythonTypedDict {
                name: String::from("BTable"),
                properties: vec![PythonDictProperty {
                    name: String::from("column_one"),
                    nullable: false,
                    data_type: PythonDataType::String,
                }],
            },
        ];

        let result = write_python_dicts_to_str(dicts, MinimumPythonVersion::Python3_6);

        let expected = indoc! {"
            import datetime
            from typing import Any, Optional, TypedDict


            ATable = TypedDict('ATable', {
                '1column': Optional[str]
            })

            BTable = TypedDict('BTable', {
                'column_one': str
            })"};

        assert_eq!(result, expected)
    }
}

use std::collections::HashMap;

use convert_case::{Case, Casing};
use itertools::Itertools;

use crate::{
    db_introspector::TableColumnDefinition,
    python_types::{PythonDictProperty, PythonTypedDict},
};

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

pub(crate) fn write_python_dicts_to_str(
    dicts: Vec<PythonTypedDict>,
    backwards_compat_forced: bool,
) -> String {
    let mut result = String::from("import datetime\nfrom typing import TypedDict, Any\n\n\n");

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

            let requires_backwards_compat =
                iter.any(|p| starts_with_number(p) || contains_space(p) || is_python_keyword(p));

            if backwards_compat_forced || requires_backwards_compat {
                dict.as_backwards_compat_typed_dict_class_str()
            } else {
                dict.as_typed_dict_class_str()
            }
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

        let result = write_python_dicts_to_str(dict, false);
        let expected = indoc! {"
            import datetime
            from typing import TypedDict, Any


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

        let result = write_python_dicts_to_str(dicts, false);
        let expected = indoc! {"
            import datetime
            from typing import TypedDict, Any


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

        let result = write_python_dicts_to_str(dicts, false);
        let expected = indoc! {"
            import datetime
            from typing import TypedDict, Any


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

        let result = write_python_dicts_to_str(dicts, false);
        let expected = indoc! {"
            import datetime
            from typing import TypedDict, Any


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

        let result = write_python_dicts_to_str(dicts, false);

        let expected = indoc! {"
            import datetime
            from typing import TypedDict, Any


            ATable = TypedDict('ATable', {
                '1column': str | None,
            })

            class BTable(TypedDict):
                column_one: str
            "};

        assert_eq!(result, expected)
    }

    #[test]
    fn backwards_compat_forced_true() {
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

        let result = write_python_dicts_to_str(dicts, true);

        let expected = indoc! {"
            import datetime
            from typing import TypedDict, Any


            ATable = TypedDict('ATable', {
                '1column': str | None,
            })

            BTable = TypedDict('BTable', {
                'column_one': str,
            })"};

        assert_eq!(result, expected)
    }
}

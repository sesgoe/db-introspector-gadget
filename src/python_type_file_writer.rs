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
        .sorted_by_key(|f| f.name.clone())
        .map(|dict| {
            let mut iter = dict.properties.iter();

            let starts_with_number =
                |p: &PythonDictProperty| p.name.chars().next().unwrap().is_numeric();
            let contains_space = |p: &PythonDictProperty| p.name.contains(' ');
            let contains_keyword = |p: &PythonDictProperty| p.name == "from";

            let requires_backwards_compat =
                iter.any(|p| starts_with_number(p) || contains_space(p) || contains_keyword(p));

            if requires_backwards_compat || backwards_compat_forced {
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

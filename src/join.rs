use std::collections::HashMap;

// struct Table(
//     name: String,
//     columns: Vec<String>,
//     df: DataFrame,
// )

// impl Table {
//     pub fn new(name: String, columns: Vec<String>, df: DataFrame) -> Self {
//         Table { name, columns, df }
//     }
// }

// pub fn create_tables() {}

pub fn parse_columns(
    tables_names: &[String],
    columns_strings: &[String],
) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize each label with an empty vector
    for table_name in tables_names {
        result.insert(table_name.clone(), Vec::new());
    }

    // Process each column string
    for column_string in columns_strings {
        for part in column_string.split('=') {
            if let Some((label, column)) = part.split_once('.') {
                if let Some(columns) = result.get_mut(label) {
                    columns.push(column.to_string());
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_columns() {
        let labels = vec!["T1".to_string(), "T2".to_string(), "T3".to_string()];
        let column_strings = vec![
            "T1.col11=T2.col12=T3.col13".to_string(),
            "T1.col21=T2.col22=T3.col23".to_string(),
        ];

        let result = parse_columns(&labels, &column_strings);

        assert_eq!(result["T1"], vec!["col11", "col21"]);
        assert_eq!(result["T2"], vec!["col12", "col22"]);
        assert_eq!(result["T3"], vec!["col13", "col23"]);
    }
}

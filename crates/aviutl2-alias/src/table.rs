use crate::FromTableValue;

/// テーブル構造を定義します。
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Table {
    items: indexmap::IndexMap<String, TableItem>,
}

#[derive(Clone, PartialEq, Eq)]
struct TableItem {
    value: Option<String>,
    table: Option<Table>,
}

impl Table {
    /// 空のテーブルを作成します。
    pub fn new() -> Self {
        Self {
            items: indexmap::IndexMap::new(),
        }
    }

    /// 指定したキーに値を挿入します。
    pub fn insert_value<T: std::fmt::Display>(&mut self, key: &str, value: T) {
        self.items
            .entry(key.to_string())
            .or_insert_with(|| TableItem {
                value: None,
                table: None,
            })
            .value = Some(value.to_string());
    }
    /// 指定したキーにサブテーブルを挿入します。
    pub fn insert_table(&mut self, key: &str, table: Table) {
        self.items
            .entry(key.to_string())
            .or_insert_with(|| TableItem {
                value: None,
                table: None,
            })
            .table = Some(table);
    }
    /// 指定したキーの値を削除します。
    pub fn remove_value(&mut self, key: &str) {
        if let Some(item) = self.items.get_mut(key) {
            item.value = None;
            if item.table.is_none() {
                self.items.shift_remove(key);
            }
        }
    }
    /// 指定したキーのサブテーブルを削除します。
    pub fn remove_table(&mut self, key: &str) {
        if let Some(item) = self.items.get_mut(key) {
            item.table = None;
            if item.value.is_none() {
                self.items.shift_remove(key);
            }
        }
    }
    /// 指定したキーの値を文字列として読み取ります。
    pub fn get_value(&self, key: &str) -> Option<&String> {
        self.items.get(key).and_then(|item| item.value.as_ref())
    }

    /// 指定したキーの値をパースして読み取ります。
    pub fn parse_value<T: FromTableValue>(&self, key: &str) -> Option<Result<T, T::Err>> {
        self.get_value(key)
            .map(|value_str| T::from_table_value(value_str))
    }
    /// 指定したキーの値への可変参照を取得します。
    pub fn get_value_mut(&mut self, key: &str) -> Option<&mut String> {
        self.items.get_mut(key).and_then(|item| item.value.as_mut())
    }
    /// 指定したキーのサブテーブルを取得します。
    pub fn get_table(&self, key: &str) -> Option<&Table> {
        self.items.get(key).and_then(|item| item.table.as_ref())
    }
    /// 指定したキーのサブテーブルへの可変参照を取得します。
    pub fn get_table_mut(&mut self, key: &str) -> Option<&mut Table> {
        self.items.get_mut(key).and_then(|item| item.table.as_mut())
    }

    /// 別のテーブルをマージします。
    pub fn merge(&mut self, other: &Table) {
        for (key, other_item) in &other.items {
            match self.items.get_mut(key) {
                Some(item) => {
                    if let Some(other_value) = &other_item.value {
                        item.value = Some(other_value.clone());
                    }
                    if let Some(other_table) = &other_item.table {
                        if let Some(item_table) = &mut item.table {
                            item_table.merge(other_table);
                        } else {
                            item.table = Some(other_table.clone());
                        }
                    }
                }
                None => {
                    self.items.insert(key.clone(), other_item.clone());
                }
            }
        }
    }

    /// 値を列挙するイテレーターを返します。
    pub fn values<'a>(&'a self) -> TableValuesIterator<'a> {
        TableValuesIterator::new(self)
    }

    /// 可変参照で値を列挙します。
    pub fn values_mut<'a>(&'a mut self) -> TableValuesIteratorMut<'a> {
        TableValuesIteratorMut::new(self)
    }

    /// 値が空かどうかを返します。
    pub fn is_values_empty(&self) -> bool {
        self.items.values().all(|item| item.value.is_none())
    }

    /// 子テーブルを列挙するイテレーターを返します。
    pub fn subtables<'a>(&'a self) -> SubTablesIterator<'a> {
        SubTablesIterator::new(self)
    }

    /// 子テーブルを可変参照で列挙します。
    pub fn subtables_mut<'a>(&'a mut self) -> SubTablesIteratorMut<'a> {
        SubTablesIteratorMut::new(self)
    }

    /// 子テーブルが空かどうかを返します。
    pub fn is_subtables_empty(&self) -> bool {
        self.items.values().all(|item| item.table.is_none())
    }

    /// `0`、`1`、`2`...のキーを持つ子テーブルを配列として列挙するイテレーターを返します。
    pub fn iter_subtables_as_array<'a>(&'a self) -> ArraySubTablesIterator<'a> {
        ArraySubTablesIterator::new(self)
    }

    // pub fn iter_subtables_as_array_mut<'a>(&'a mut self) -> ArraySubTablesIteratorMut<'a> {
    //     ArraySubTablesIteratorMut::new(self)
    // }

    /// テーブルを文字列として書き出します。
    ///
    /// `prefix`はサブテーブルの名前の接頭辞として使用されます。
    /// 具体的には、`${prefix}.${key}`の形式でサブテーブルの名前が生成されます。
    pub fn write_table(
        &self,
        f: &mut impl std::fmt::Write,
        prefix: Option<&str>,
    ) -> std::fmt::Result {
        for (key, item) in self.values() {
            write!(f, "{}={}\r\n", key, item)?;
        }
        let prefix = prefix.map_or("".to_string(), |p| format!("{}.", p));
        for (key, sub_table) in self.subtables() {
            let subtable_name = format!("{}{}", prefix, key);
            if !sub_table.is_values_empty() {
                write!(f, "[{}]\r\n", subtable_name)?;
            }
            sub_table.write_table(f, Some(&subtable_name))?;
        }
        Ok(())
    }
}

/// [`Table::values`]で使われるイテレーター。
#[derive(Debug)]
pub struct TableValuesIterator<'a> {
    table: &'a Table,
    index: usize,
}
impl<'a> TableValuesIterator<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self { table, index: 0 }
    }
}
impl<'a> Iterator for TableValuesIterator<'a> {
    type Item = (&'a String, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.table.items.len() {
            let item = &self.table.items.get_index(self.index).unwrap();
            self.index += 1;
            if let Some(value) = &item.1.value {
                return Some((item.0, value));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.items.len().saturating_sub(self.index);
        (0, Some(remaining))
    }
}

/// [`Table::values_mut`]で使われるイテレーター。
pub struct TableValuesIteratorMut<'a> {
    inner: indexmap::map::IterMut<'a, String, TableItem>,
}
impl<'a> TableValuesIteratorMut<'a> {
    pub fn new(table: &'a mut Table) -> Self {
        Self {
            inner: table.items.iter_mut(),
        }
    }
}
impl<'a> Iterator for TableValuesIteratorMut<'a> {
    type Item = (&'a String, &'a mut String);

    fn next(&mut self) -> Option<Self::Item> {
        for (key, item) in self.inner.by_ref() {
            if let Some(value) = item.value.as_mut() {
                return Some((key, value));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.inner.len();
        (0, Some(remaining))
    }
}

/// [`Table::subtables`]で使われるイテレーター。
pub struct SubTablesIterator<'a> {
    table: &'a Table,
    index: usize,
}
impl<'a> SubTablesIterator<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self { table, index: 0 }
    }
}
impl<'a> Iterator for SubTablesIterator<'a> {
    type Item = (&'a String, &'a Table);
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.table.items.len() {
            let item = &self.table.items.get_index(self.index).unwrap();
            self.index += 1;
            if let Some(sub_table) = &item.1.table {
                return Some((item.0, sub_table));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.items.len().saturating_sub(self.index);
        (0, Some(remaining))
    }
}

/// [`Table::subtables_mut`]で使われるイテレーター。
pub struct SubTablesIteratorMut<'a> {
    inner: indexmap::map::IterMut<'a, String, TableItem>,
}
impl<'a> SubTablesIteratorMut<'a> {
    pub fn new(table: &'a mut Table) -> Self {
        Self {
            inner: table.items.iter_mut(),
        }
    }
}
impl<'a> Iterator for SubTablesIteratorMut<'a> {
    type Item = (&'a String, &'a mut Table);
    fn next(&mut self) -> Option<Self::Item> {
        for (key, item) in self.inner.by_ref() {
            if let Some(sub_table) = item.table.as_mut() {
                return Some((key, sub_table));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.inner.len();
        (0, Some(remaining))
    }
}

/// [`Table::iter_subtables_as_array`]で使われるイテレーター。
pub struct ArraySubTablesIterator<'a> {
    table: &'a Table,
    index: usize,
}
impl<'a> ArraySubTablesIterator<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self { table, index: 0 }
    }
}
impl<'a> Iterator for ArraySubTablesIterator<'a> {
    type Item = &'a Table;
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.index.to_string();
        self.index += 1;
        if let Some(sub_table) = self.table.get_table(&key) {
            Some(sub_table)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.items.len().saturating_sub(self.index);
        (0, Some(remaining))
    }
}

// TODO: コメントを解除する（有識者求む）
// pub struct ArraySubTablesIteratorMut<'a> {
//     index: usize,
//     table: &'a mut Table,
// }
// impl<'a> ArraySubTablesIteratorMut<'a> {
//     pub fn new(table: &'a mut Table) -> Self {
//         Self {
//             index: 0,
//             table,
//         }
//     }
// }
// impl<'a> Iterator for ArraySubTablesIteratorMut<'a> {
//     type Item = &'a mut Table;
//     fn next(&mut self) -> Option<Self::Item> {
//         let raw: *mut Table = self.table;
//         let key = self.index.to_string();
//         self.index += 1;
//
//         // Safety: &mut self.tableは他に存在しないはず
//         unsafe {
//             match (&mut *raw).get_table_mut(&key) {
//                 Some(sub) => Some(sub),
//                 None => None,
//             }
//         }
//     }
// }

/// テーブルのパースエラー。
#[derive(Debug, Clone, thiserror::Error)]
pub enum TableParseError {
    #[error("Invalid line: {0}")]
    InvalidLine(String),
}

impl std::fmt::Debug for TableItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableItem")
            .field("value", &self.value)
            .field("table", &self.table)
            .finish()
    }
}

impl std::str::FromStr for Table {
    type Err = TableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut root = Table::new();
        let mut current_path: Vec<String> = Vec::new();

        for line in s.lines() {
            if line.trim().is_empty() {
                continue;
            } else if line.starts_with('[') && line.ends_with(']') {
                let section = &line[1..line.len() - 1];
                current_path.clear();
                if !section.is_empty() {
                    current_path.extend(section.split('.').map(|part| part.to_string()));
                }
            } else if let Some((key, value)) = line.split_once('=') {
                let target = ensure_path(&mut root, &current_path);
                target.insert_value(key, value);
            } else {
                return Err(TableParseError::InvalidLine(line.to_string()));
            }
        }

        Ok(root)
    }
}

fn ensure_path<'a>(mut table: &'a mut Table, path: &[String]) -> &'a mut Table {
    for segment in path {
        let entry = table
            .items
            .entry(segment.clone())
            .or_insert_with(|| TableItem {
                value: None,
                table: Some(Table::new()),
            });
        table = entry.table.get_or_insert_with(Table::new);
    }
    table
}
impl std::fmt::Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field(
                "values",
                &self.values().collect::<indexmap::IndexMap<_, _>>(),
            )
            .field(
                "subtables",
                &self.subtables().collect::<indexmap::IndexMap<_, _>>(),
            )
            .finish()
    }
}
impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_table(f, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_table_insert_and_get() {
        let mut table = Table::new();
        table.insert_value("key1", "value1");
        assert_eq!(table.get_value("key1"), Some(&"value1".to_string()));
        let mut sub_table = Table::new();
        sub_table.insert_value("sub_key1", "sub_value1");
        table.insert_table("sub_table", sub_table.clone());
        assert_eq!(table.get_table("sub_table"), Some(&sub_table));
    }

    #[test]
    fn test_parse_table() {
        let input = include_str!("../test_assets/tracks.aup2");
        let table: Table = input.parse().unwrap();

        let (project_table_name, project_table) = table.subtables().next().unwrap();
        assert_eq!(project_table_name, "project");
        assert_eq!(
            project_table.get_value("version"),
            Some(&"2001802".to_string())
        );

        assert_eq!(
            table
                .get_table("0")
                .unwrap()
                .get_table("0")
                .unwrap()
                .get_value("effect.name"),
            Some(&"test_tracks".to_string())
        );
        assert_eq!(
            table
                .get_table("2")
                .unwrap()
                .get_table("1")
                .unwrap()
                .get_value("effect.name"),
            Some(&"標準描画".to_string())
        );

        let layers = table
            .iter_subtables_as_array()
            .map(|t| t.parse_value::<usize>("layer").unwrap().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(layers, vec![0, 1, 2]);

        insta::assert_debug_snapshot!(table);
        assert_eq!(table.to_string(), input);
    }

    #[test]
    fn test_values_mut_iterator() {
        let mut table = Table::new();
        table.insert_value("key1", "value1");
        table.insert_value("key2", "value2");

        for (_key, value) in table.values_mut() {
            value.push_str("_mutated");
        }

        assert_eq!(table.get_value("key1"), Some(&"value1_mutated".to_string()));
        assert_eq!(table.get_value("key2"), Some(&"value2_mutated".to_string()));
    }

    #[test]
    fn test_subtables_mut_iterator() {
        let mut table = Table::new();
        let mut sub = Table::new();
        sub.insert_value("inner", "value");
        table.insert_table("sub1", sub);

        for (_key, sub_table) in table.subtables_mut() {
            sub_table.insert_value("updated", "true");
        }

        assert_eq!(
            table.get_table("sub1").unwrap().get_value("updated"),
            Some(&"true".to_string())
        );
    }
}

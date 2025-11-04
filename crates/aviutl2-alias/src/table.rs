/// エイリアスファイルで使用されるテーブル構造を定義します。
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Table {
    items: indexmap::IndexMap<String, TableItem>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TableItem {
    value: Option<String>,
    table: Option<Table>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            items: indexmap::IndexMap::new(),
        }
    }

    pub fn insert_value<T: std::fmt::Display>(&mut self, key: &str, value: T) {
        self.items
            .entry(key.to_string())
            .or_insert_with(|| TableItem {
                value: None,
                table: None,
            })
            .value = Some(value.to_string());
    }
    pub fn insert_table(&mut self, key: &str, table: Table) {
        self.items
            .entry(key.to_string())
            .or_insert_with(|| TableItem {
                value: None,
                table: None,
            })
            .table = Some(table);
    }
    pub fn remove_value(&mut self, key: &str) {
        if let Some(item) = self.items.get_mut(key) {
            item.value = None;
            if item.table.is_none() {
                self.items.shift_remove(key);
            }
        }
    }
    pub fn remove_table(&mut self, key: &str) {
        if let Some(item) = self.items.get_mut(key) {
            item.table = None;
            if item.value.is_none() {
                self.items.shift_remove(key);
            }
        }
    }
    pub fn get_value(&self, key: &str) -> Option<&String> {
        self.items.get(key).and_then(|item| item.value.as_ref())
    }
    pub fn get_value_mut(&mut self, key: &str) -> Option<&mut String> {
        self.items.get_mut(key).and_then(|item| item.value.as_mut())
    }
    pub fn get_table(&self, key: &str) -> Option<&Table> {
        self.items.get(key).and_then(|item| item.table.as_ref())
    }
    pub fn get_table_mut(&mut self, key: &str) -> Option<&mut Table> {
        self.items.get_mut(key).and_then(|item| item.table.as_mut())
    }

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

    pub fn values_iter<'a>(&'a self) -> TableValuesIterator<'a> {
        TableValuesIterator::new(self)
    }

    pub fn subtables_iter<'a>(&'a self) -> SubTablesIterator<'a> {
        SubTablesIterator::new(self)
    }
}

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
}

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
}

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
        let mut current_table_keys = vec![];
        let mut current_table = Table::new();
        let mut tables = vec![];
        let mut appeared_index = indexmap::IndexSet::new();
        for line in s.lines() {
            if line.starts_with("[") && line.ends_with("]") {
                let table_name = &line[1..line.len() - 1];
                let new_table_keys: Vec<&str> = table_name.split('.').collect();
                for (i, key) in new_table_keys.iter().enumerate() {
                    appeared_index.insert((i, *key));
                }
                tables.push((current_table_keys, current_table));
                current_table = Table::new();
                current_table_keys = new_table_keys;
            } else if let Some((key, value)) = line.split_once('=') {
                current_table.insert_value(key, value);
            } else if line.trim().is_empty() {
                continue;
            } else {
                return Err(TableParseError::InvalidLine(line.to_string()));
            }
        }
        tables.push((current_table_keys, current_table));
        tables.sort_by_key(|a| {
            a.0.iter()
                .enumerate()
                .map(|(i, key)| {
                    appeared_index
                        .get_index_of(&(i, *key))
                        .unwrap_or(usize::MAX)
                })
                .collect::<Vec<usize>>()
        });
        let root_table = tables.remove(0);
        let mut table_tree = vec![("<root>".to_string(), root_table.1)];
        for (keys, table) in tables {
            let last_common_key_index = table_tree
                .iter()
                .enumerate()
                .position(|(i, (name, _))| {
                    if i >= keys.len() {
                        return true;
                    }
                    if i == 0 {
                        return false;
                    }
                    name != keys[i - 1]
                })
                .unwrap_or(keys.len())
                - 1;
            for _ in (last_common_key_index + 1)..table_tree.len() {
                let (name, tbl) = table_tree.pop().unwrap();
                let parent = &mut table_tree.last_mut().unwrap();
                let parent_table = &mut parent.1;
                parent_table.insert_table(&name, tbl);
            }
            for key in &keys[last_common_key_index..(keys.len() - 1)] {
                table_tree.push((key.to_string(), Table::new()));
            }
            table_tree.push((keys.last().unwrap().to_string(), table));
        }
        while table_tree.len() > 1 {
            let (name, tbl) = table_tree.pop().unwrap();
            let parent_table = &mut table_tree.last_mut().unwrap().1;
            parent_table.insert_table(&name, tbl);
        }
        Ok(table_tree.pop().unwrap().1)
    }
}
impl std::fmt::Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field(
                "values",
                &self.values_iter().collect::<indexmap::IndexMap<_, _>>(),
            )
            .field(
                "subtables",
                &self.subtables_iter().collect::<indexmap::IndexMap<_, _>>(),
            )
            .finish()
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

        let (project_table_name, project_table) = table.subtables_iter().next().unwrap();
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

        insta::assert_debug_snapshot!(table);
    }
}

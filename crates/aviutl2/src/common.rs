pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

pub(crate) fn format_file_filters(file_filters: Vec<FileFilter>) -> String {
    let mut file_filter = String::new();
    for filter in file_filters {
        if !file_filter.is_empty() {
            file_filter.push('\x00');
        }
        file_filter.push_str(&filter.name);
        file_filter.push('\x00');
        file_filter.push_str(
            &filter
                .extensions
                .iter()
                .map(|ext| format!("*.{}", ext))
                .collect::<Vec<_>>()
                .join(";"),
        );
        file_filter.push('\x00');
    }

    return file_filter;
}

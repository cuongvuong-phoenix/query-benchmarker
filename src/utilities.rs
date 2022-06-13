use std::path::Path;

pub fn is_sql_script(path: impl AsRef<Path>) -> bool {
    if let Some(extension) = path.as_ref().extension() {
        extension == "sql"
    } else {
        false
    }
}

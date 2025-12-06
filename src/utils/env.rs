use std::collections::HashMap;

pub fn generate_export_script(vars: &HashMap<String, String>) -> String {
    vars.iter()
        .map(|(key, value)| format!("export \"{key}={value}\""))
        .collect::<Vec<_>>()
        .join(";")
}

pub fn generate_unset_script(vars: &HashMap<String, String>) -> String {
    vars.keys()
        .map(|key| format!("unset \"{key}\""))
        .collect::<Vec<_>>()
        .join(";")
}

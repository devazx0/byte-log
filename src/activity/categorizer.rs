use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Categorizer {
    rules: HashMap<String, String>,
}

impl Categorizer {
    pub fn new(rules: HashMap<String, String>) -> Self {
        let mut normalized = HashMap::new();
        for (k, v) in rules {
            normalized.insert(k.to_lowercase(), v);
        }
        Self { rules: normalized }
    }

    pub fn categorize(&self, executable_name: &str) -> String {
        let key = executable_name.to_lowercase();
        if let Some(cat) = self.rules.get(&key) {
            return cat.clone();
        }

        // Try stripping extension (e.g. "code.exe" -> "code")
        if let Some(stem) = key.strip_suffix(".exe") {
            if let Some(cat) = self.rules.get(stem) {
                return cat.clone();
            }
        }

        "Other".to_string()
    }
}

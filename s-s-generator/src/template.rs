use std::collections::HashMap;

use crate::frontmatter::FrontMatter;

pub fn apply_template(template: &str, front_matter: &FrontMatter, content: &str) -> String {
    let mut values = HashMap::new();
    values.insert("content".to_string(), content.to_string());

    if let Some(title) = &front_matter.title {
        values.insert("title".to_string(), title.clone());
    }
    if let Some(date) = &front_matter.date {
        values.insert("date".to_string(), date.clone());
    }

    let mut output = template.to_string();
    for (key, value) in values {
        let placeholder = format!("{{{{{key}}}}}");
        output = output.replace(&placeholder, &value);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_placeholders() {
        let template = "<html><title>{{title}}</title><body>{{content}}</body></html>";
        let front_matter = FrontMatter {
            title: Some("Test".to_string()),
            date: None,
        };

        let result = apply_template(template, &front_matter, "<p>Hi</p>");
        assert!(result.contains("<title>Test</title>"));
        assert!(result.contains("<p>Hi</p>"));
        assert!(!result.contains("{{title}}"));
    }
}

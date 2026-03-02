use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Content,
    Structure,
    Metadata,
    Title,
    Links,
    Images,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Content => "content",
            ChangeType::Structure => "structure",
            ChangeType::Metadata => "metadata",
            ChangeType::Title => "title",
            ChangeType::Links => "links",
            ChangeType::Images => "images",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub details: Option<String>,
}

pub fn detect_changes(html_content: &str, _old_hash: &str, new_hash: &str) -> Vec<Change> {
    let mut changes = Vec::new();
    let document = Html::parse_document(html_content);
    
    // Check title
    let title_selector = Selector::parse("title").unwrap();
    if let Some(title) = document.select(&title_selector).next() {
        let title_text = title.text().collect::<String>().trim().to_string();
        if !title_text.is_empty() {
            changes.push(Change {
                id: uuid::Uuid::new_v4().to_string(),
                change_type: ChangeType::Title,
                old_value: None,
                new_value: Some(title_text),
                details: Some("Page title detected".to_string()),
            });
        }
    }
    
    // Check meta tags (description, keywords)
    let meta_selector = Selector::parse("meta").unwrap();
    let mut meta_changes = Vec::new();
    
    for element in document.select(&meta_selector) {
        if let Some(name) = element.value().attr("name") {
            if name == "description" || name == "keywords" {
                if let Some(content) = element.value().attr("content") {
                    meta_changes.push(format!("{}: {}", name, content));
                }
            }
        }
    }
    
    if !meta_changes.is_empty() {
        changes.push(Change {
            id: uuid::Uuid::new_v4().to_string(),
            change_type: ChangeType::Metadata,
            old_value: None,
            new_value: Some(meta_changes.join(", ")),
            details: Some("Meta tags detected".to_string()),
        });
    }
    
    // Count links
    let link_selector = Selector::parse("a").unwrap();
    let link_count = document.select(&link_selector).count();
    
    changes.push(Change {
        id: uuid::Uuid::new_v4().to_string(),
        change_type: ChangeType::Links,
        old_value: None,
        new_value: Some(link_count.to_string()),
        details: Some("Link count".to_string()),
    });
    
    // Count images
    let img_selector = Selector::parse("img").unwrap();
    let img_count = document.select(&img_selector).count();
    
    changes.push(Change {
        id: uuid::Uuid::new_v4().to_string(),
        change_type: ChangeType::Images,
        old_value: None,
        new_value: Some(img_count.to_string()),
        details: Some("Image count".to_string()),
    });
    
    // Content-based change (hash difference indicates content change)
    changes.push(Change {
        id: uuid::Uuid::new_v4().to_string(),
        change_type: ChangeType::Content,
        old_value: None,
        new_value: Some(new_hash.to_string()),
        details: Some("Content hash changed".to_string()),
    });
    
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_changes_basic() {
        let html = r#"<html><head><title>Test</title></head><body><a href="link">Link</a></body></html>"#;
        let changes = detect_changes(html, "old_hash", "new_hash");
        
        assert!(!changes.is_empty());
        
        // Should have title change
        let title_change = changes.iter().find(|c| c.change_type == ChangeType::Title);
        assert!(title_change.is_some());
    }

    #[test]
    fn test_change_type_serialization() {
        let change_type = ChangeType::Content;
        let serialized = change_type.as_str();
        assert_eq!(serialized, "content");
    }
}

use percent_encoding::percent_decode;
use std::collections::HashMap;
use std::hash::BuildHasher;
use unicode_segmentation::UnicodeSegmentation;

use errors::Result;

/// Get word count and estimated reading time
pub fn get_reading_analytics(content: &str) -> (usize, usize) {
    let word_count: usize = content.unicode_words().count();

    // https://help.medium.com/hc/en-us/articles/214991667-Read-time
    // 275 seems a bit too high though
    (word_count, ((word_count + 199) / 200))
}

pub fn check_page_for_anchor(url: &str, body: &String) -> errors::Result<()> {
    // find the #, or if there's no #, assume `url` is the anchor name without preceeding #
    let index = match url.find('#') {
        Some(i) => i,
        None => 0,
    };
    let anchor = url.get(index + 1..).unwrap();
    let checks = [
        format!(" id={}", anchor),
        format!(" ID={}", anchor),
        format!(" id='{}'", anchor),
        format!(" ID='{}'", anchor),
        format!(r#" id="{}""#, anchor),
        format!(r#" ID="{}""#, anchor),
        format!(" name={}", anchor),
        format!(" NAME={}", anchor),
        format!(" name='{}'", anchor),
        format!(" NAME='{}'", anchor),
        format!(r#" name="{}""#, anchor),
        format!(r#" NAME="{}""#, anchor),
    ];

    if checks.iter().any(|check| body[..].contains(&check[..])) {
        Ok(())
    } else {
        Err(errors::Error::from(format!("Anchor `#{}` not found on page", anchor)))
    }
}

/// Result of a successful resolution of an internal link.
#[derive(Debug, PartialEq, Clone)]
pub struct ResolvedInternalLink {
    /// Resolved link target, as absolute URL address.
    pub permalink: String,
    /// Internal path to the .md file, without the leading `@/`.
    pub md_path: String,
    /// Optional anchor target.
    /// We can check whether it exists only after all the markdown rendering is done.
    pub anchor: Option<String>,
}

/// Resolves an internal link (of the `@/posts/something.md#hey` sort) to its absolute link and
/// returns the path + anchor as well
pub fn resolve_internal_link<S: BuildHasher>(
    link: &str,
    permalinks: &HashMap<String, String, S>,
) -> Result<ResolvedInternalLink> {
    // First we remove the ./ since that's zola specific
    let clean_link = link.replacen("@/", "", 1);
    // Then we remove any potential anchor
    // parts[0] will be the file path and parts[1] the anchor if present
    let parts = clean_link.split('#').collect::<Vec<_>>();
    // If we have slugification turned off, we might end up with some escaped characters so we need
    // to decode them first
    let decoded = percent_decode(parts[0].as_bytes()).decode_utf8_lossy().to_string();
    let target =
        permalinks.get(&decoded).ok_or_else(|| format!("Relative link {} not found.", link))?;
    if parts.len() > 1 {
        Ok(ResolvedInternalLink {
            permalink: format!("{}#{}", target, parts[1]),
            md_path: decoded,
            anchor: Some(parts[1].to_string()),
        })
    } else {
        Ok(ResolvedInternalLink { permalink: target.to_string(), md_path: decoded, anchor: None })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{check_page_for_anchor, get_reading_analytics, resolve_internal_link};

    #[test]
    fn can_resolve_valid_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("@/pages/about.md", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_valid_root_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("@/about.md", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_internal_links_with_anchors() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("@/pages/about.md#hello", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about#hello");
        assert_eq!(res.md_path, "pages/about.md".to_string());
        assert_eq!(res.anchor, Some("hello".to_string()));
    }

    #[test]
    fn can_resolve_escaped_internal_links() {
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "pages/about space.md".to_string(),
            "https://vincent.is/about%20space/".to_string(),
        );
        let res = resolve_internal_link("@/pages/about%20space.md#hello", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about%20space/#hello");
        assert_eq!(res.md_path, "pages/about space.md".to_string());
        assert_eq!(res.anchor, Some("hello".to_string()));
    }

    #[test]
    fn errors_resolve_inexistant_internal_link() {
        let res = resolve_internal_link("@/pages/about.md#hello", &HashMap::new());
        assert!(res.is_err());
    }

    #[test]
    fn reading_analytics_empty_text() {
        let (word_count, reading_time) = get_reading_analytics("  ");
        assert_eq!(word_count, 0);
        assert_eq!(reading_time, 0);
    }

    #[test]
    fn reading_analytics_short_text() {
        let (word_count, reading_time) = get_reading_analytics("Hello World");
        assert_eq!(word_count, 2);
        assert_eq!(reading_time, 1);
    }

    #[test]
    fn reading_analytics_long_text() {
        let mut content = String::new();
        for _ in 0..1000 {
            content.push_str(" Hello world");
        }
        let (word_count, reading_time) = get_reading_analytics(&content);
        assert_eq!(word_count, 2000);
        assert_eq!(reading_time, 10);
    }

    #[test]
    fn can_validate_anchors_with_double_quotes() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, &body);
        assert!(res.is_ok());
    }

    // https://github.com/getzola/zola/issues/948
    #[test]
    fn can_validate_anchors_in_capital() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 ID="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, &body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_with_single_quotes() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = "<body><h3 id='method.collect'>collect</h3></body>".to_string();
        let res = check_page_for_anchor(url, &body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_without_quotes() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = "<body><h3 id=method.collect>collect</h3></body>".to_string();
        let res = check_page_for_anchor(url, &body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_with_name_attr() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 name="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, &body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_fail_when_anchor_not_found() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#me";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, &body);
        assert!(res.is_err());
    }
}

// Per-request options for customizing individual API calls.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Options that customize individual API requests.
///
/// Use with [`OpenAI::with_options()`](crate::OpenAI::with_options) to create
/// a client clone that applies these options to every request:
///
/// ```ignore
/// use openai_oxide::RequestOptions;
///
/// let custom = client.with_options(
///     RequestOptions::new()
///         .header("X-Custom", "value")
///         .timeout(Duration::from_secs(30))
/// );
/// let response = custom.chat().completions().create(request).await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    /// Extra headers to include in the request.
    pub headers: Option<HeaderMap>,

    /// Extra query parameters to append to the URL.
    pub query: Option<Vec<(String, String)>>,

    /// Extra JSON fields to merge into the request body (JSON requests only).
    pub extra_body: Option<serde_json::Value>,

    /// Override the request timeout.
    pub timeout: Option<Duration>,
}

impl RequestOptions {
    /// Create empty request options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single header.
    #[must_use]
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let map = self.headers.get_or_insert_with(HeaderMap::new);
        if let (Ok(n), Ok(v)) = (
            name.as_ref().parse::<HeaderName>(),
            value.as_ref().parse::<HeaderValue>(),
        ) {
            map.insert(n, v);
        }
        self
    }

    /// Set multiple headers at once (replaces any previously set headers).
    #[must_use]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Add a single query parameter.
    #[must_use]
    pub fn query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query
            .get_or_insert_with(Vec::new)
            .push((key.into(), value.into()));
        self
    }

    /// Set all query parameters (replaces any previously set).
    #[must_use]
    pub fn query(mut self, params: Vec<(String, String)>) -> Self {
        self.query = Some(params);
        self
    }

    /// Set extra JSON fields to merge into the request body.
    #[must_use]
    pub fn extra_body(mut self, value: serde_json::Value) -> Self {
        self.extra_body = Some(value);
        self
    }

    /// Override the request timeout.
    #[must_use]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Merge two options. Values from `other` take precedence on conflict.
    #[must_use]
    pub fn merge(&self, other: &RequestOptions) -> RequestOptions {
        RequestOptions {
            headers: merge_headers(&self.headers, &other.headers),
            query: merge_query(&self.query, &other.query),
            extra_body: merge_json(&self.extra_body, &other.extra_body),
            timeout: other.timeout.or(self.timeout),
        }
    }

    /// Returns true if no options are set.
    pub fn is_empty(&self) -> bool {
        self.headers.is_none()
            && self.query.is_none()
            && self.extra_body.is_none()
            && self.timeout.is_none()
    }
}

/// Merge two optional header maps. `b` values win on key collision.
fn merge_headers(a: &Option<HeaderMap>, b: &Option<HeaderMap>) -> Option<HeaderMap> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), Some(b)) => {
            let mut merged = a.clone();
            for (key, value) in b.iter() {
                merged.insert(key.clone(), value.clone());
            }
            Some(merged)
        }
    }
}

/// Merge two optional query param vecs. Both are appended; `b` comes after `a`.
fn merge_query(
    a: &Option<Vec<(String, String)>>,
    b: &Option<Vec<(String, String)>>,
) -> Option<Vec<(String, String)>> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), Some(b)) => {
            let mut merged = a.clone();
            merged.extend(b.iter().cloned());
            Some(merged)
        }
    }
}

/// Deep-merge two optional JSON values. `b` fields win on conflict.
fn merge_json(
    a: &Option<serde_json::Value>,
    b: &Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), Some(b)) => Some(deep_merge_value(a.clone(), b.clone())),
    }
}

/// Recursively merge two JSON values. Object fields from `b` override `a`.
fn deep_merge_value(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match (a, b) {
        (Value::Object(mut a_map), Value::Object(b_map)) => {
            for (k, v) in b_map {
                let merged = match a_map.remove(&k) {
                    Some(existing) => deep_merge_value(existing, v),
                    None => v,
                };
                a_map.insert(k, merged);
            }
            Value::Object(a_map)
        }
        // Non-object: b wins
        (_, b) => b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let opts = RequestOptions::new();
        assert!(opts.is_empty());
    }

    #[test]
    fn test_header_builder() {
        let opts = RequestOptions::new()
            .header("X-Custom", "value1")
            .header("X-Other", "value2");
        let headers = opts.headers.unwrap();
        assert_eq!(headers.get("X-Custom").unwrap(), "value1");
        assert_eq!(headers.get("X-Other").unwrap(), "value2");
    }

    #[test]
    fn test_query_param_builder() {
        let opts = RequestOptions::new()
            .query_param("foo", "bar")
            .query_param("baz", "qux");
        let query = opts.query.unwrap();
        assert_eq!(query.len(), 2);
        assert_eq!(query[0], ("foo".to_string(), "bar".to_string()));
    }

    #[test]
    fn test_timeout_builder() {
        let opts = RequestOptions::new().timeout(Duration::from_secs(30));
        assert_eq!(opts.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_extra_body_builder() {
        let opts = RequestOptions::new().extra_body(serde_json::json!({"key": "val"}));
        assert_eq!(opts.extra_body.unwrap(), serde_json::json!({"key": "val"}));
    }

    #[test]
    fn test_merge_empty() {
        let a = RequestOptions::new();
        let b = RequestOptions::new();
        let merged = a.merge(&b);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_merge_headers_b_wins() {
        let a = RequestOptions::new().header("X-A", "1").header("X-B", "2");
        let b = RequestOptions::new().header("X-A", "overridden");
        let merged = a.merge(&b);
        let headers = merged.headers.unwrap();
        assert_eq!(headers.get("X-A").unwrap(), "overridden");
        assert_eq!(headers.get("X-B").unwrap(), "2");
    }

    #[test]
    fn test_merge_query_appends() {
        let a = RequestOptions::new().query_param("a", "1");
        let b = RequestOptions::new().query_param("b", "2");
        let merged = a.merge(&b);
        let query = merged.query.unwrap();
        assert_eq!(query.len(), 2);
    }

    #[test]
    fn test_merge_timeout_b_wins() {
        let a = RequestOptions::new().timeout(Duration::from_secs(10));
        let b = RequestOptions::new().timeout(Duration::from_secs(30));
        let merged = a.merge(&b);
        assert_eq!(merged.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_merge_timeout_a_kept_when_b_none() {
        let a = RequestOptions::new().timeout(Duration::from_secs(10));
        let b = RequestOptions::new();
        let merged = a.merge(&b);
        assert_eq!(merged.timeout, Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_deep_merge_json() {
        let a = serde_json::json!({"x": 1, "nested": {"a": 1, "b": 2}});
        let b = serde_json::json!({"y": 2, "nested": {"b": 99, "c": 3}});
        let merged = deep_merge_value(a, b);
        assert_eq!(merged["x"], 1);
        assert_eq!(merged["y"], 2);
        assert_eq!(merged["nested"]["a"], 1);
        assert_eq!(merged["nested"]["b"], 99);
        assert_eq!(merged["nested"]["c"], 3);
    }
}

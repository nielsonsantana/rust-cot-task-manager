use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use tower::{Layer, Service};
use cot::request::Request;
use cot::response::Response;
use http::uri::{Uri, PathAndQuery};
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LocaleStrategy {
    Header,
    Path,
}

#[derive(Clone)]
pub struct LocaleMiddleware {
    supported_locales: Vec<String>,
    fallback: String,
    strategy: LocaleStrategy,
    path_mapping: HashMap<String, String>,
}

impl Default for LocaleMiddleware {
    fn default() -> Self {
        Self {
            supported_locales: Vec::new(),
            fallback: "en".to_string(),
            strategy: LocaleStrategy::Header,
            path_mapping: HashMap::new(),
        }
    }
}

impl LocaleMiddleware {
    // pub fn new() -> Self {
    //     Self::default()
    // }
    pub fn with_locales(locales: Vec<&'static str>) -> Self {
        Self {
            supported_locales: locales.into_iter().map(|s| s.to_string()).collect(),
            fallback: "en".to_string(),
            strategy: LocaleStrategy::Header,
            path_mapping: HashMap::new(),
        }
    }

    pub fn with_default_locale(mut self, locale: &str) -> Self {
        self.fallback = locale.to_string();
        self
    }

    pub fn with_strategy(mut self, strategy: LocaleStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_path_mapping(mut self, mapping: impl AsRef<[(&'static str, &'static str)]>) -> Self {
        self.path_mapping = mapping
            .as_ref()
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.to_string()))
            .collect();
        self
    }
}

impl<S> Layer<S> for LocaleMiddleware {
    type Service = LocaleService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LocaleService { 
            inner,
            supported_locales: self.supported_locales.clone(),
            fallback: self.fallback.clone(),
            strategy: self.strategy,
            path_mapping: self.path_mapping.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LocaleService<S> {
    inner: S,
    supported_locales: Vec<String>,
    fallback: String,
    strategy: LocaleStrategy,
    path_mapping: HashMap<String, String>,
}

fn parse_accept_language(header: &str) -> Vec<(String, f32)> {
    let mut langs: Vec<(String, f32)> = header
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|part| {
            let mut parts = part.split(";q=");
            let tag = parts.next().unwrap_or("").trim().to_lowercase();
            let q = parts.next().and_then(|q| q.parse::<f32>().ok()).unwrap_or(1.0);
            (tag, q)
        })
        .collect();

    langs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    langs
}

fn match_locale_fuzzy(requested: &str, supported: &[String]) -> Option<String> {
    for sup in supported {
        if sup.to_lowercase() == requested {
            return Some(sup.clone());
        }
    }

    let req_base = requested.split('-').next().unwrap_or("").to_lowercase();
    for sup in supported {
        let sup_base = sup.to_lowercase();
        let sup_base = sup_base.split('-').next().unwrap_or("");
        if req_base == sup_base {
            return Some(sup.clone());
        }
    }

    None
}

fn determine_locale_from_referer(referer: &str, mapping: &HashMap<String, String>) -> Option<String> {
    let uri = Uri::from_str(referer).ok()?;
    let path = uri.path();
    let segments: Vec<&str> = path.split('/').collect();

    if segments.len() >= 2 && !segments[1].is_empty() {
        return mapping.get(&segments[1].to_lowercase()).cloned();
    }
    None
}

fn determine_locale_from_header(
    referer: Option<&str>,
    accept_lang: Option<&str>, 
    supported: &[String], 
    fallback: &str,
    path_mapping: &HashMap<String, String>
) -> String {
    if let Some(r) = referer {
        if let Some(matched) = determine_locale_from_referer(r, path_mapping) {
            return matched;
        }
    }

    if let Some(h) = accept_lang {
        let parsed = parse_accept_language(h);
        for (lang, _) in parsed {
            if let Some(matched) = match_locale_fuzzy(&lang, supported) {
                return matched;
            }
        }
    }

    fallback.to_string()
}

impl<S> Service<Request> for LocaleService<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let locale = match self.strategy {
            LocaleStrategy::Header => {
                let referer = req.headers().get("referer").and_then(|h| h.to_str().ok());
                let accept_lang = req.headers().get("accept-language").and_then(|h| h.to_str().ok());
                
                determine_locale_from_header(
                    referer, 
                    accept_lang, 
                    &self.supported_locales, 
                    &self.fallback,
                    &self.path_mapping
                )
            },
            LocaleStrategy::Path => {
                // 1. Check Referer first for priority
                let referer = req.headers().get("referer").and_then(|h| h.to_str().ok());
                let referer_locale = referer.and_then(|r| determine_locale_from_referer(r, &self.path_mapping));

                // 2. Check current Path
                let path = req.uri().path();
                let segments: Vec<&str> = path.split('/').collect();
                let mut path_locale = None;

                if segments.len() >= 3 && segments[0].is_empty() {
                    let requested_locale = segments[1].to_lowercase();
                    
                    if let Some(matched) = self.path_mapping.get(&requested_locale) {
                        path_locale = Some(matched.clone());

                        // Still rewrite path even if Referer is the winner, 
                        // so the router receives a clean path.
                        let mut new_segments = segments.clone();
                        new_segments.remove(1);
                        
                        let mut new_path = new_segments.join("/");
                        if new_path.is_empty() {
                            new_path = "/".to_string();
                        }

                        let new_paq = if let Some(query) = req.uri().query() {
                            format!("{}?{}", new_path, query)
                        } else {
                            new_path
                        };

                        if let Ok(path_and_query) = PathAndQuery::from_str(&new_paq) {
                            let mut parts = req.uri().clone().into_parts();
                            parts.path_and_query = Some(path_and_query);
                            if let Ok(new_uri) = Uri::from_parts(parts) {
                                *req.uri_mut() = new_uri;
                            }
                        }
                    }
                }
                
                // Priority: Referer > Path > Fallback
                referer_locale.or(path_locale).unwrap_or_else(|| self.fallback.clone())
            }
        };
        
        rust_i18n::set_locale(&locale);

        let mut inner = self.inner.clone();
        Box::pin(async move {
            inner.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_accept_language() {
        let result = parse_accept_language("pt-BR,pt;q=0.9,en;q=0.8");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "pt-br");
        assert_eq!(result[0].1, 1.0);
    }

    #[test]
    fn test_match_locale_fuzzy() {
        let supported = vec!["en".to_string(), "pt-BR".to_string()];
        assert_eq!(match_locale_fuzzy("pt", &supported), Some("pt-BR".to_string()));
        assert_eq!(match_locale_fuzzy("PT-BR", &supported), Some("pt-BR".to_string()));
    }

    #[test]
    fn test_referer_priority_in_header_strategy() {
        let mut mapping = HashMap::new();
        mapping.insert("pt".to_string(), "pt-BR".to_string());
        
        let supported = vec!["en".to_string(), "pt-BR".to_string()];
        let referer = Some("https://example.com/pt/dashboard");
        let accept_lang = Some("en-US,en;q=0.9");
        
        let result = determine_locale_from_header(referer, accept_lang, &supported, "en", &mapping);
        assert_eq!(result, "pt-BR");
    }

    #[test]
    fn test_path_strategy_referer_priority() {
        // Mock setup for path mapping
        let mut path_mapping = HashMap::new();
        path_mapping.insert("pt".to_string(), "pt-BR".to_string());
        path_mapping.insert("en".to_string(), "en-US".to_string());

        // Scenario: Request path is /api, but Referer is /pt/home
        // Referer should win.
        let referer_url = "https://example.com/pt/home";
        let referer_locale = determine_locale_from_referer(referer_url, &path_mapping);
        assert_eq!(referer_locale, Some("pt-BR".to_string()));
        
        // Scenario: Request path is /api, no Referer
        let path_segments = vec!["", "api"];
        let path_locale = path_mapping.get(path_segments[1]).cloned();
        assert_eq!(path_locale, None);
    }

    #[test]
    fn test_path_reconstruction() {
        let segments = vec!["", "pt", "home", ""];
        let mut new_segments = segments.clone();
        new_segments.remove(1);
        let new_path = new_segments.join("/");
        assert_eq!(new_path, "/home/");

        let segments_root = vec!["", "pt", ""];
        let mut new_segments_root = segments_root.clone();
        new_segments_root.remove(1);
        let new_path_root = new_segments_root.join("/");
        assert_eq!(new_path_root, "/");
    }
}
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use tower::{Layer, Service};
use cot::request::Request;
use cot::response::Response;

#[derive(Clone)]
pub struct LocaleMiddleware {
    supported_locales: Vec<String>,
    fallback: String,
}

impl Default for LocaleMiddleware {
    fn default() -> Self {
        Self {
            supported_locales: Vec::new(),
            fallback: "en".to_string(),
        }
    }
}

impl LocaleMiddleware {
    pub fn new() -> Self {
        Self::default()
    }

    #[warn(dead_code)]
    pub fn with_locales(locales: Vec<&'static str>) -> Self {
        Self {
            supported_locales: locales.into_iter().map(|s| s.to_string()).collect(),
            fallback: "en".to_string(),
        }
    }
}

impl<S> Layer<S> for LocaleMiddleware {
    type Service = LocaleService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LocaleService { 
            inner,
            supported_locales: self.supported_locales.clone(),
            fallback: self.fallback.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LocaleService<S> {
    inner: S,
    supported_locales: Vec<String>,
    fallback: String,
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

    // Sort descending by weight
    langs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    langs
}

fn match_locale(requested: &str, supported: &[String]) -> Option<String> {
    // Pass 1: Direct case-insensitive match
    for sup in supported {
        if sup.to_lowercase() == requested {
            return Some(sup.clone());
        }
    }

    // Pass 2: Base language match (e.g., "pt" matches "pt-BR")
    let req_base = requested.split('-').next().unwrap_or("");
    for sup in supported {
        let sup_base = sup.to_lowercase();
        let sup_base = sup_base.split('-').next().unwrap_or("");
        if req_base == sup_base {
            return Some(sup.clone());
        }
    }

    None
}

fn determine_locale(header: Option<&str>, supported: &[String], fallback: &str) -> String {
    if let Some(h) = header {
        let parsed = parse_accept_language(h);
        for (lang, _) in parsed {
            if let Some(matched) = match_locale(&lang, supported) {
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

    fn call(&mut self, req: Request) -> Self::Future {
        let header_val = req.headers().get("accept-language").and_then(|h| h.to_str().ok());
        let locale = determine_locale(header_val, &self.supported_locales, &self.fallback);
        
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
        assert_eq!(result[1].0, "pt");
        assert_eq!(result[1].1, 0.9);
        assert_eq!(result[2].0, "en");
        assert_eq!(result[2].1, 0.8);
    }

    #[test]
    fn test_match_locale_direct() {
        let supported = vec!["en".to_string(), "pt-BR".to_string()];
        assert_eq!(match_locale("pt-br", &supported), Some("pt-BR".to_string()));
        assert_eq!(match_locale("en", &supported), Some("en".to_string()));
    }

    #[test]
    fn test_match_locale_base() {
        let supported = vec!["en".to_string(), "pt-BR".to_string()];
        assert_eq!(match_locale("pt", &supported), Some("pt-BR".to_string()));
    }

    #[test]
    fn test_determine_locale() {
        let supported = vec!["en".to_string(), "pt-BR".to_string()];
        
        assert_eq!(determine_locale(Some("pt-BR,en;q=0.9"), &supported, "en"), "pt-BR");
        assert_eq!(determine_locale(Some("en-US,en"), &supported, "en"), "en");
        assert_eq!(determine_locale(Some("fr-FR"), &supported, "en"), "en");
        assert_eq!(determine_locale(None, &supported, "en"), "en");
    }
}
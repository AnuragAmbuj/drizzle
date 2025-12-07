use domain::route::{PathMatch, Route};
use log::debug;
use pingora::proxy::Session;
use snapshot::Snapshot;

pub struct Router;

impl Router {
    pub fn match_request<'a>(snapshot: &'a Snapshot, session: &Session) -> Option<&'a Route> {
        let req_header = session.req_header();
        let path = req_header.uri.path();

        let host = req_header
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();

        debug!("Router matching request: Host={}, Path={}", host, path);

        // Sorting by priority (descending) should ideally be done once at snapshot load time.
        // For MVP we can iterate or sort on the fly (expensive but safe).
        // Since we return reference, we can't easily sort the vector in place without &mut.
        // We will linear scan and keep the best match based on priority.

        // Assumption: Higher priority value wins.
        let mut best_match: Option<&Route> = None;

        for route in &snapshot.routes {
            // 1. Match Headers (e.g. Host)
            let mut headers_match = true;
            for (key, required_val) in &route.match_headers {
                if key.to_lowercase() == "host" {
                    if host != required_val {
                        headers_match = false;
                        break;
                    }
                } else {
                    match req_header.headers.get(key).and_then(|v| v.to_str().ok()) {
                        Some(val) if val == required_val => continue,
                        _ => {
                            headers_match = false;
                            break;
                        }
                    }
                }
            }
            if !headers_match {
                continue;
            }

            // 2. Match Path
            let path_match = match &route.match_path {
                PathMatch::Prefix(prefix) => path.starts_with(prefix),
                PathMatch::Exact(exact) => path == exact,
                PathMatch::Regex(re_str) => {
                    // Compiling regex on every request is bad.
                    // For MVP we probably skip regex or warn about performance.
                    // Let's implement prefix/exact first properly.
                    match regex::Regex::new(re_str) {
                        Ok(re) => re.is_match(path),
                        Err(_) => false,
                    }
                }
            };

            if !path_match {
                continue;
            }

            // 3. Match Methods
            if !route.match_methods.is_empty() {
                let method = req_header.method.as_str();
                if !route.match_methods.iter().any(|m| m == method) {
                    continue;
                }
            }

            // Found a match! Check priority.
            match best_match {
                None => best_match = Some(route),
                Some(current) => {
                    if route.priority > current.priority {
                        best_match = Some(route);
                    }
                }
            }
        }

        best_match
    }
}

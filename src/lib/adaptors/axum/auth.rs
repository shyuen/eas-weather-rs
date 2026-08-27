use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use subtle::ConstantTimeEq;

/// Returns the `Bearer` token from an `Authorization` header, if present.
///
/// Only `Authorization: Bearer <token>` is accepted for now. Invalid or
/// missing values yield `None`, which the middleware treats as unauthorized.
fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    (!token.is_empty()).then_some(token)
}

/// Constant-time equality check for two strings.
///
/// Lengths are allowed to differ; a timing attack on the length of an API key
/// is not practical, and this keeps the comparison usable for arbitrary keys.
fn constant_time_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Middleware guarding the `/conf` route group.
///
/// When an API key is configured, every request must present
/// `Authorization: Bearer <api-key>`; otherwise the request is rejected with
/// `401 Unauthorized` and a `WWW-Authenticate: Bearer` challenge. When no key
/// is configured the routes stay open (the default/dev behaviour).
pub async fn require_api_key(
    State(api_key): State<Option<String>>,
    request: Request,
    next: Next,
) -> Response {
    match &api_key {
        Some(expected) => match bearer_token(request.headers()) {
            Some(provided) if constant_time_eq(provided, expected) => next.run(request).await,
            _ => unauthorized(),
        },
        None => next.run(request).await,
    }
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"))],
        "unauthorized: a valid API key is required",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use tower::ServiceExt;

    async fn status(app: Router, req: Request<Body>) -> u16 {
        app.oneshot(req).await.unwrap().status().as_u16()
    }

    fn app(api_key: Option<String>) -> Router {
        Router::new()
            .route("/conf/app", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                api_key,
                require_api_key,
            ))
    }

    fn bearer(key: &str) -> Request<Body> {
        Request::builder()
            .uri("/conf/app")
            .header(AUTHORIZATION, format!("Bearer {key}"))
            .body(Body::empty())
            .unwrap()
    }

    fn plain() -> Request<Body> {
        Request::builder()
            .uri("/conf/app")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn open_when_no_key_configured() {
        assert_eq!(status(app(None), plain()).await, 200);
    }

    #[tokio::test]
    async fn open_with_valid_key() {
        let app = app(Some("secret".into()));
        assert_eq!(status(app, bearer("secret")).await, 200);
    }

    #[tokio::test]
    async fn unauthorized_without_header() {
        let app = app(Some("secret".into()));
        assert_eq!(status(app, plain()).await, 401);
    }

    #[tokio::test]
    async fn unauthorized_with_wrong_key() {
        let app = app(Some("secret".into()));
        assert_eq!(status(app, bearer("nope")).await, 401);
    }

    #[tokio::test]
    async fn unauthorized_with_empty_bearer() {
        let app = app(Some("secret".into()));
        let req = Request::builder()
            .uri("/conf/app")
            .header(AUTHORIZATION, "Bearer")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status(app, req).await, 401);
    }

    #[tokio::test]
    async fn unauthorized_without_bearer_scheme() {
        let app = app(Some("secret".into()));
        let req = Request::builder()
            .uri("/conf/app")
            .header(AUTHORIZATION, "Basic dXNlcjpwYXNz")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status(app, req).await, 401);
    }
}

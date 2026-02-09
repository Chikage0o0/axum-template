use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response;

#[cfg(embed_frontend)]
use mime_guess::from_path;

#[cfg(embed_frontend)]
use rust_embed::RustEmbed;

#[cfg(embed_frontend)]
#[derive(RustEmbed)]
#[folder = "frontend/build"]
struct FrontendAssets;

#[cfg(any(embed_frontend, test))]
const INDEX_FILE: &str = "index.html";

#[cfg(embed_frontend)]
pub async fn serve_frontend_index() -> Response {
    build_embedded_response(INDEX_FILE)
}

#[cfg(not(embed_frontend))]
pub async fn serve_frontend_index() -> Response {
    frontend_not_enabled_response()
}

#[cfg(embed_frontend)]
pub async fn serve_frontend_path(Path(path): Path<String>) -> Response {
    let requested = resolve_requested_asset_path(&path).unwrap_or_else(|| INDEX_FILE.to_string());

    if FrontendAssets::get(&requested).is_some() {
        return build_embedded_response(&requested);
    }

    build_embedded_response(INDEX_FILE)
}

#[cfg(not(embed_frontend))]
pub async fn serve_frontend_path(_: Path<String>) -> Response {
    frontend_not_enabled_response()
}

#[cfg(any(embed_frontend, test))]
fn resolve_requested_asset_path(path: &str) -> Option<String> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Some(INDEX_FILE.to_string());
    }

    if trimmed
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return None;
    }

    let filename = trimmed.rsplit('/').next()?;
    if !filename.contains('.') {
        return Some(INDEX_FILE.to_string());
    }

    Some(trimmed.to_string())
}

#[cfg(embed_frontend)]
fn build_embedded_response(path: &str) -> Response {
    let Some(content) = FrontendAssets::get(path) else {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )
            .body(Body::from("前端静态资源缺失，请先执行构建"))
            .unwrap_or_else(|_| Response::new(Body::from("前端静态资源缺失，请先执行构建")));
    };

    let mime = from_path(path).first_or_octet_stream();

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime.as_ref())
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
        )
        .body(Body::from(content.data.into_owned()))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

#[cfg(not(embed_frontend))]
fn frontend_not_enabled_response() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )
        .body(Body::from("开发模式未启用前端嵌入，请使用前端开发服务器"))
        .unwrap_or_else(|_| {
            Response::new(Body::from("开发模式未启用前端嵌入，请使用前端开发服务器"))
        })
}

#[cfg(test)]
mod tests {
    use super::resolve_requested_asset_path;

    #[test]
    fn should_resolve_root_to_index_html() {
        assert_eq!(
            resolve_requested_asset_path(""),
            Some("index.html".to_string())
        );
    }

    #[test]
    fn should_keep_static_asset_path() {
        assert_eq!(
            resolve_requested_asset_path("assets/app.js"),
            Some("assets/app.js".to_string())
        );
    }

    #[test]
    fn should_block_parent_path_segments() {
        assert_eq!(resolve_requested_asset_path("../secret.txt"), None);
    }

    #[test]
    fn should_fallback_route_like_path_to_index_html() {
        assert_eq!(
            resolve_requested_asset_path("settings"),
            Some("index.html".to_string())
        );
    }
}

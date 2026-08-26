use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub title: String,
    pub detail: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Http(reqwest::Error),
    #[error("API error ({status}): {}", .errors.first().map(|e| e.title.clone()).unwrap_or_default())]
    Api { status: u16, errors: Vec<ApiError> },
    #[error("timed out waiting for action to complete")]
    Timeout,
    #[error("failed to parse response: {0}")]
    Parse(String),
    #[error("{0}")]
    Aborted(String),
    #[error("config error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize)]
struct ErrorEnvelope {
    errors: Vec<ApiError>,
}

pub fn parse_api_errors(status: u16, body: &str) -> Error {
    match serde_json::from_str::<ErrorEnvelope>(body) {
        Ok(env) if !env.errors.is_empty() => Error::Api {
            status,
            errors: env.errors,
        },
        _ => Error::Api {
            status,
            errors: vec![ApiError {
                title: body.to_string(),
                detail: None,
            }],
        },
    }
}

pub fn map_progenitor<E: serde::Serialize + std::fmt::Debug>(
    e: progenitor_client::Error<E>,
) -> Error {
    use progenitor_client::Error as Pe;
    match e {
        Pe::CommunicationError(e) => Error::Http(e),
        Pe::ErrorResponse(rv) => {
            let status = rv.status().as_u16();
            let body = serde_json::to_string(&rv.into_inner()).unwrap_or_default();
            parse_api_errors(status, &body)
        }
        other => Error::Parse(format!("{other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jsonapi_error_body() {
        let body = r#"{"errors":[{"title":"Bad Request","detail":"bad channel"}]}"#;
        match parse_api_errors(400, body) {
            Error::Api { status, errors } => {
                assert_eq!(status, 400);
                assert_eq!(errors[0].title, "Bad Request");
                assert_eq!(errors[0].detail.as_deref(), Some("bad channel"));
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }

    #[test]
    fn falls_back_on_non_jsonapi_body() {
        match parse_api_errors(500, "internal failure") {
            Error::Api { status, errors } => {
                assert_eq!(status, 500);
                assert_eq!(errors[0].title, "internal failure");
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }
}

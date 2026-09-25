//! Failures as the protocol carries them: a tool result with `isError` and one text content,
//! the JSON `{ "code": "…", "params": { … } }` — stable codes an agent reacts to, never
//! sentences.

use life_pixel_service::CodedError;
use life_pixel_service::library::LibraryError;
use rmcp::ErrorData;
use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::{Map, Value, json};

/// Arguments that do not match the tool's schema: service's code.
pub(crate) const REQUEST_MALFORMED: &str = "request.malformed";
/// An animation the caller does not have.
const ANIMATION_NOT_FOUND: &str = "library.animation_not_found";

/// `request.malformed`, without parameters.
pub(crate) fn malformed() -> CodedError {
    CodedError {
        code: REQUEST_MALFORMED,
        params: Map::new(),
    }
}

/// `service.unavailable`: what never happens with valid data — a value that does not serialize.
pub(crate) fn unavailable() -> CodedError {
    LibraryError::Unavailable.into()
}

/// The JSON of `error`: its code and its parameters.
pub(crate) fn to_json(error: &CodedError) -> Value {
    json!({ "code": error.code, "params": error.params })
}

/// The failed tool result of `error`.
pub(crate) fn tool_failure(error: &CodedError) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(to_json(error).to_string())])
}

/// The protocol error of a failed resource read — reads have no `isError` —, the code and its
/// parameters as the error's data.
pub(crate) fn resource_failure(error: &CodedError) -> ErrorData {
    let data = Some(to_json(error));
    match error.code {
        ANIMATION_NOT_FOUND => ErrorData::resource_not_found(error.code, data),
        REQUEST_MALFORMED => ErrorData::invalid_params(error.code, data),
        _ => ErrorData::invalid_request(error.code, data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_is_one_text_content_holding_the_code_and_its_params() {
        let mut params = Map::new();
        params.insert("max".to_owned(), json!(1000));
        let error = CodedError {
            code: "draw.too_many_operations",
            params,
        };

        let result = tool_failure(&error);

        assert_eq!(result.is_error, Some(true));
        let [content] = result.content.as_slice() else {
            panic!("one content expected");
        };
        let ContentBlock::Text(text) = content else {
            panic!("a text content expected");
        };
        let value: Value = serde_json::from_str(&text.text).unwrap();
        assert_eq!(
            value,
            json!({ "code": "draw.too_many_operations", "params": { "max": 1000 } })
        );
    }
}

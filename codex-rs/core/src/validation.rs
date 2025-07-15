use crate::api_logger::{ApiLogger, TS_FORMAT};
use serde_json::Value;
use time::OffsetDateTime;
use crate::ResponseItem;

/// Validate that after any assistant tool call in chat messages, the next message is a tool response.
pub async fn validate_chat_message_sequence(
    messages: &[Value],
    api_logger: Option<&ApiLogger>,
) {
    for (idx, msg) in messages.iter().enumerate() {
        if msg.get("tool_calls").is_some() {
            let valid_next = messages.get(idx + 1)
                .map(|next| next.get("role").and_then(|r| r.as_str()) == Some("tool"))
                .unwrap_or(false);
            if !valid_next {
                if let Some(logger) = api_logger {
                    let entry = serde_json::json!({
                        "ts": OffsetDateTime::now_utc().format(TS_FORMAT).unwrap_or_default(),
                        "type": "error",
                        "message": "Request has assistant tool call without subsequent tool response",
                        "index": idx,
                        "messages": messages,
                    });
                    let _ = logger.log(&entry).await;
                }
                panic!(
                    "Invalid request: assistant tool call at index {} not followed by tool response",
                    idx
                );
            }
        }
    }
}

/// Validate that any tool call in prompt input is followed by the corresponding output, and no unresolved calls remain.
pub async fn validate_response_input_sequence(
    input: &[ResponseItem],
    api_logger: Option<&ApiLogger>,
) {
    for i in 0..input.len() {
        match &input[i] {
            ResponseItem::FunctionCall { call_id, .. } => {
                let valid = input.get(i + 1)
                    .map(|next| matches!(next,
                        ResponseItem::FunctionCallOutput { call_id: cid, .. } if cid == call_id
                    ))
                    .unwrap_or(false);
                if !valid {
                    if let Some(logger) = api_logger {
                        let entry = serde_json::json!({
                            "ts": OffsetDateTime::now_utc().format(TS_FORMAT).unwrap_or_default(),
                            "type": "error",
                            "message": "Responses API payload input has tool call without output",
                            "index": i,
                            "call_id": call_id,
                        });
                        let _ = logger.log(&entry).await;
                    }
                    panic!(
                        "Invalid request: FunctionCall at input index {} not followed by FunctionCallOutput",
                        i
                    );
                }
            }
            ResponseItem::LocalShellCall { id, .. } if id.is_some() => {
                if let Some(call_id) = id {
                    let valid = input.get(i + 1)
                        .map(|next| matches!(next,
                            ResponseItem::FunctionCallOutput { call_id: cid, .. } if cid == call_id
                        ))
                        .unwrap_or(false);
                    if !valid {
                        if let Some(logger) = api_logger {
                            let entry = serde_json::json!({
                                "ts": OffsetDateTime::now_utc().format(TS_FORMAT).unwrap_or_default(),
                                "type": "error",
                                "message": "Responses API payload input has local shell call without output",
                                "index": i,
                                "call_id": call_id,
                            });
                            let _ = logger.log(&entry).await;
                        }
                        panic!(
                            "Invalid request: LocalShellCall at input index {} not followed by FunctionCallOutput",
                            i
                        );
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(item) = input.last() {
        match item {
            ResponseItem::FunctionCall { .. } | ResponseItem::LocalShellCall { .. } => {
                if let Some(logger) = api_logger {
                    let entry = serde_json::json!({
                        "ts": OffsetDateTime::now_utc().format(TS_FORMAT).unwrap_or_default(),
                        "type": "error",
                        "message": "Responses API request ends with unanswered tool call",
                        "unresolved_call": format!("{:?}", item),
                    });
                    let _ = logger.log(&entry).await;
                }
                panic!(
                    "Invalid request: ends with unanswered tool call in prompt.input"
                );
            }
            _ => {}
        }
    }
}

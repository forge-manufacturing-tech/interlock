
use super::*;

#[test]
fn test_parse_single_action() {
    let input = r#"
Thought: I should read the file.
Action: read_file
Action Input: { "blob_id": "123" }
"#;
    let (action, val, processed) = parse_agent_response(input).unwrap();
    assert_eq!(action, "read_file");
    assert_eq!(val["blob_id"], "123");
    assert!(processed.contains("Action: read_file"));
    assert!(processed.contains("123"));
}

#[test]
fn test_parse_multiple_actions() {
    let input = r#"
Action: action_one
Action Input: { "key": "value_one" }
---
Thought: Now I do the second thing.
Action: action_two
Action Input: { "key": "value_two" }
"#;
    let (action, val, processed) = parse_agent_response(input).unwrap();
    assert_eq!(action, "action_one");
    assert_eq!(val["key"], "value_one");

    // Ensure processed text does NOT contain the second action
    assert!(!processed.contains("action_two"));
    assert!(!processed.contains("value_two"));

    // Ensure processed text stops at "---" or before
    // processed text should include the JSON input for action_one
    assert!(processed.contains("value_one"));
}

#[test]
fn test_parse_complex_json() {
        let input = r#"
Action: complex_tool
Action Input: {
"nested": {
    "array": [1, 2, 3]
},
"str": "value"
}
Final Answer: I am done.
"#;
    let (action, val, processed) = parse_agent_response(input).unwrap();
    assert_eq!(action, "complex_tool");
    assert_eq!(val["nested"]["array"][0], 1);
    assert!(!processed.contains("Final Answer"));
}

#[test]
fn test_parse_markdown_json() {
    let input = r#"
Action: tool
Action Input: ```json
{ "a": 1 }
```
"#;
    let (action, val, _) = parse_agent_response(input).unwrap();
    assert_eq!(action, "tool");
    assert_eq!(val["a"], 1);
}

#[test]
fn test_parse_no_action() {
    let input = "Just a thought.";
    assert!(parse_agent_response(input).is_none());
}

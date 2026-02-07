use super::*;

#[test]
fn test_layout_component_type_serializes_list_item_as_camel_case() {
    let json = serde_json::to_string(&LayoutComponentType::ListItem).unwrap();
    assert_eq!(json, "\"listItem\"");
}

#[test]
fn test_layout_component_type_deserializes_legacy_listitem_value() {
    let parsed: LayoutComponentType = serde_json::from_str("\"listitem\"").unwrap();
    assert_eq!(parsed, LayoutComponentType::ListItem);
}

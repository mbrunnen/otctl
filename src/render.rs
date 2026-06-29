use comfy_table::Table;
use serde_json::Value;

pub fn to_output(value: &Value, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    }
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Object(map) => {
            let mut table = Table::new();
            table.set_header(vec!["field", "value"]);
            for (k, v) in map {
                table.add_row(vec![k.clone(), scalarise(v)]);
            }
            table.to_string()
        }
        Value::Array(items) => items.iter().map(scalarise).collect::<Vec<_>>().join("\n"),
    }
}

fn scalarise(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_flag_emits_pretty_json() {
        let v = serde_json::json!({"state":"leader"});
        assert!(to_output(&v, true).contains("\"state\": \"leader\""));
    }

    #[test]
    fn scalar_renders_bare() {
        assert_eq!(to_output(&serde_json::json!("leader"), false), "leader");
    }

    #[test]
    fn object_renders_key_values() {
        let v = serde_json::json!({"rloc16":"0x4400","state":"leader"});
        let out = to_output(&v, false);
        assert!(out.contains("rloc16"));
        assert!(out.contains("0x4400"));
    }
}

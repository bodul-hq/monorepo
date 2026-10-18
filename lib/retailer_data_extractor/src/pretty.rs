//! Pretty-printing JSON text without reordering its keys.

/// Re-indents valid JSON text the way `serde_json::to_string_pretty` lays it out.
///
/// Parsing into a `Value` would sort object keys, as `serde_json` is built here without
/// `preserve_order`, and enabling that feature would change key order for every crate that
/// shares the dependency. Working on the text keeps each document in the order its stage wrote
/// it.
pub fn pretty_json(json: &str) -> String {
    fn newline(out: &mut String, depth: usize) {
        out.push('\n');
        out.extend(std::iter::repeat_n("  ", depth));
    }

    let mut out = String::with_capacity(json.len() * 2);
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = json.chars().peekable();

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '{' | '[' => {
                out.push(c);
                let close = if c == '{' { '}' } else { ']' };
                while chars.next_if(char::is_ascii_whitespace).is_some() {}
                // An empty object or array stays on one line.
                if chars.next_if_eq(&close).is_some() {
                    out.push(close);
                } else {
                    depth += 1;
                    newline(&mut out, depth);
                }
            }
            '}' | ']' => {
                depth -= 1;
                newline(&mut out, depth);
                out.push(c);
            }
            ',' => {
                out.push(c);
                newline(&mut out, depth);
            }
            ':' => out.push_str(": "),
            c if c.is_ascii_whitespace() => {}
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::pretty_json;
    use html_extractor::serde_json::{self, Value};

    #[test]
    fn lays_out_like_serde_json() {
        // Keys already sorted, so `Value` keeps the order and the layouts can be compared.
        let json = r#"{"a":{},"b":[1,{"w":null,"x":"a,b:{c}\\\"[ ]"}],"c":[],"d":-1500.25}"#;
        let expected =
            serde_json::to_string_pretty(&serde_json::from_str::<Value>(json).unwrap()).unwrap();

        assert_eq!(pretty_json(json), expected);
    }

    #[test]
    fn keeps_keys_in_arrival_order() {
        assert_eq!(
            pretty_json(r#"{"b":1,"a":2}"#),
            "{\n  \"b\": 1,\n  \"a\": 2\n}"
        );
    }
}

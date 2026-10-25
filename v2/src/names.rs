use fancy_regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;
struct Tables {
    data: Value,
    es5: Regex,
    es6: Regex,
    reserved: Regex,
    words: Regex,
}
fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let data: Value = serde_json::from_str(include_str!("tables.json")).expect("pinned tables");
        let re =
            |key: &str| Regex::new(data[key].as_str().expect("regex table")).expect("pinned regex");
        let (es5, es6, reserved, words) = (re("es5"), re("es6"), re("reserved"), re("words"));
        Tables {
            data,
            es5,
            es6,
            reserved,
            words,
        }
    })
}
pub(crate) fn units(s: &str) -> String {
    s.encode_utf16()
        .map(|u| {
            char::from_u32(if (0xd800..=0xdfff).contains(&u) {
                0xf0000 + u32::from(u)
            } else {
                u32::from(u)
            })
            .expect("mapped unit")
        })
        .collect()
}
pub(crate) fn decode_units(s: &str) -> Result<String, String> {
    let utf16: Vec<u16> = s
        .chars()
        .flat_map(|c| {
            let n = u32::from(c);
            if (0xfd800..=0xfdfff).contains(&n) {
                vec![(n - 0xf0000) as u16]
            } else {
                let mut b = [0; 2];
                c.encode_utf16(&mut b).to_vec()
            }
        })
        .collect();
    String::from_utf16(&utf16).map_err(|_| "Unpaired Unicode surrogate in generated output".into())
}
pub(crate) fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    let Some(c) = chars.next() else {
        return String::new();
    };
    let up = tables().data["upper"][u32::from(c).to_string()]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| c.to_string());
    up + chars.as_str()
}
fn numeric(s: &str) -> bool {
    if s.is_empty()
        || s.starts_with(['+', '-'])
        || !s
            .bytes()
            .all(|b| b.is_ascii_hexdigit() || b"xX+-.".contains(&b))
    {
        return false;
    }
    if s.starts_with("0x") || s.starts_with("0X") {
        return !s[2..].is_empty() && s[2..].bytes().all(|c| c.is_ascii_hexdigit());
    }
    s.parse::<f64>().is_ok()
}
pub(crate) fn needs_quotes(s: &str) -> Result<bool, String> {
    // Both ES5 and ES6 must accept a name. Escaped identifiers are decoded first.
    let decoded = unescape_identifier(s);
    let decoded_es5 = if s.contains("\\u{") {
        s.to_owned()
    } else {
        decoded.clone()
    };
    let t = tables();
    let valid = t
        .es5
        .is_match(&decoded_es5)
        .map_err(|_| "Identifier complexity limit")?
        && t.es6
            .is_match(&decoded)
            .map_err(|_| "Identifier complexity limit")?;
    if valid {
        Ok(t.reserved
            .is_match(&decoded)
            .map_err(|_| "Identifier complexity limit")?
            || decoded.contains(['\u{200c}', '\u{200d}']))
    } else {
        Ok(!numeric(s))
    }
}
fn unescape_identifier(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'u') {
            chars.next();
            let brace = chars.peek() == Some(&'{');
            if brace {
                chars.next();
            }
            let mut hex = String::new();
            while let Some(&x) = chars.peek() {
                if x.is_ascii_hexdigit() && (brace || hex.len() < 4) {
                    hex.push(x);
                    chars.next();
                } else {
                    break;
                }
            }
            if brace && chars.peek() == Some(&'}') {
                chars.next();
            }
            if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                if let Some(ch) = char::from_u32(cp) {
                    out.push_str(&units(&ch.to_string()));
                    continue;
                }
            }
            out.push('\0');
        } else {
            out.push(c);
        }
    }
    out
}
pub(crate) fn interface_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Invalid JSON".into());
    }
    let base = upper_first(name);
    Ok(format!(
        "{}{base}",
        if needs_quotes(&base)? { "_" } else { "" }
    ))
}
pub(crate) fn pascal(input: &str) -> Result<String, String> {
    let t = tables();
    let mut deburred = String::new();
    for c in input.chars() {
        if matches!(u32::from(c),0x300..=0x36f|0xfe20..=0xfe2f|0x20d0..=0x20ff)
            || matches!(c, '\'' | '\u{2019}')
        {
            continue;
        }
        if let Some(x) = t.data["deburr"][c.to_string()].as_str() {
            deburred.push_str(x);
        } else {
            deburred.push(c);
        }
    }
    let mut out = String::new();
    for word in t.words.find_iter(&deburred) {
        let word = word.map_err(|_| "Word complexity limit")?;
        let word = decode_units(word.as_str())?;
        let mut chars = word.chars();
        if let Some(c) = chars.next() {
            let upper = t.data["upper"][u32::from(c).to_string()]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| c.to_string());
            out.push_str(&units(&upper));
            out.push_str(&units(chars.as_str()));
        }
    }
    Ok(out)
}

pub(crate) fn property(name: &str) -> Result<String, String> {
    if !needs_quotes(name)? {
        return Ok(name.to_owned());
    }
    let decoded = unescape_identifier(name);
    let valid_es6 = tables()
        .es6
        .is_match(&decoded)
        .map_err(|_| "Identifier complexity limit")?;
    if valid_es6 {
        Ok(format!("'{name}'"))
    } else {
        Ok(format!(
            "'{}'",
            name.replace('\\', "\\\\").replace('\'', "\\'")
        ))
    }
}

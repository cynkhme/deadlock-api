//! The drift guard shared by the fixture and live tests.

/// Walks `raw` and `back` together: a key of `raw` missing in `back`, or a
/// value in `raw` that became null/empty in `back`, is a diff. Numbers only
/// have to agree as f64 (the model may widen ints to floats).
pub fn compare_shape(
    path: &str,
    raw: &serde_json::Value,
    back: &serde_json::Value,
    diffs: &mut Vec<String>,
) {
    use serde_json::Value::*;
    match (raw, back) {
        (Object(r), Object(b)) => {
            for (k, rv) in r {
                let p = format!("{path}/{k}");
                match b.get(k) {
                    None => diffs.push(format!("{p}: key missing in the model")),
                    Some(bv) => compare_shape(&p, rv, bv, diffs),
                }
            }
        }
        (Array(r), Array(b)) => {
            if r.len() != b.len() {
                diffs.push(format!("{path}: {} elements became {}", r.len(), b.len()));
            }
            for (i, (rv, bv)) in r.iter().zip(b).enumerate() {
                compare_shape(&format!("{path}/{i}"), rv, bv, diffs);
            }
        }
        (Null, _) => {}
        (_, Null) => diffs.push(format!("{path}: value became null")),
        (Number(r), Number(b)) => {
            if r.as_f64() != b.as_f64() {
                diffs.push(format!("{path}: {r} became {b}"));
            }
        }
        (r, b) if r != b => diffs.push(format!("{path}: {r} became {b}")),
        _ => {}
    }
}

/// Decodes `raw` into `T`, serializes it back and compares the two with
/// [`compare_shape`]: empty when the model carries every field of `raw`.
pub fn drift<T: serde::de::DeserializeOwned + serde::Serialize>(
    raw: &serde_json::Value,
) -> Vec<String> {
    let decoded: T = match serde_json::from_value(raw.clone()) {
        Ok(decoded) => decoded,
        Err(e) => return vec![format!(": does not decode: {e}")],
    };
    let back = serde_json::to_value(&decoded).unwrap();
    let mut diffs = Vec::new();
    compare_shape("", raw, &back, &mut diffs);
    diffs
}

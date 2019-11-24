use plist::Value;
use std::collections::BTreeSet;

use crate::ord::OrdByKey;
// "inspired" by https://github.com/Byron/treediff-rs/blob/master/src/value/rustc_json.rs#L7

fn iterate<'a>(value: &'a Value) -> Option<Box<dyn Iterator<Item = (String, &'a Value)> + 'a>> {
    match value {
        Value::String(_)
        | Value::Boolean(_)
        | Value::Data(_)
        | Value::Date(_)
        | Value::Integer(_)
        | Value::Real(_)
        | Value::Uid(_) => None,
        Value::Array(array) => Some(Box::new(
            array.iter().enumerate().map(|(i, v)| (i.to_string(), v)),
        )),
        Value::Dictionary(dict) => Some(Box::new(dict.iter().map(|(k, v)| (k.clone(), v)))),
        Value::__Nonexhaustive => unreachable!(),
    }
}

#[derive(Debug)]
pub enum ChangeType {
    Removed(String, Value),
    Added(String, Value),
    Modified(String, Value, Value),
}

#[derive(Debug)]
pub struct Recorder {
    calls: Vec<ChangeType>,
}

impl Recorder {
    pub fn new() -> Self {
        Recorder { calls: vec![] }
    }
    pub fn modified(&mut self, key: String, from: Value, to: Value) {
        self.calls.push(ChangeType::Modified(key, from, to))
    }
    pub fn added(&mut self, key: String, new: Value) {
        self.calls.push(ChangeType::Added(key, new))
    }
    pub fn removed(&mut self, key: String, removed: Value) {
        self.calls.push(ChangeType::Removed(key, removed))
    }
}

pub fn diff(lh: &Value, rh: &Value, recorder: &mut Recorder, key: String) {
    match (iterate(&lh), iterate(&rh)) {
        (None, None) if lh == rh => {
            // Unchanged scalar
        }
        (None, None) => {
            // Modified scalar!
            recorder.modified(key, lh.clone(), rh.clone());
        }
        (Some(_), Some(_)) if lh == rh => {
            // Two objects, equal
        }
        (Some(_), None) | (None, Some(_)) => {
            // Object into scalar
            recorder.modified(key, lh.clone(), rh.clone());
        }
        (Some(li), Some(ri)) => {
            // Different objects!
            let mut sl: BTreeSet<OrdByKey<String, Value>> = BTreeSet::new();
            sl.extend(li.map(Into::into));
            let mut sr: BTreeSet<OrdByKey<String, Value>> = BTreeSet::new();
            sr.extend(ri.map(Into::into));
            for k in sr.intersection(&sl) {
                // Different
                let v1 = sl.get(k).expect("get to work");
                let v2 = sr.get(k).expect("get to work");
                let new_key = format!("{}.{}", key, v1.0);
                diff(v1.1, v2.1, recorder, new_key);
            }
            for k in sr.difference(&sl) {
                // Added
                let new_key = format!("{}.{}", key, k.0);
                recorder.added(new_key, k.1.clone());
            }
            for k in sl.difference(&sr) {
                // Removed
                let new_key = format!("{}.{}", key, k.0);
                recorder.removed(new_key, k.1.clone());
            }
        }
    }
}

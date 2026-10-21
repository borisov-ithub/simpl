pub mod string;
pub mod math;
pub mod iter;
pub mod struct_;
pub mod tuple_;
pub mod file;
pub mod datetime;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{OrderedKey, StructMap, Value};

pub fn register(globals: &mut HashMap<String, (Value, bool)>) {
    for name in ["len", "type", "str", "num", "range"] {
        globals.insert(name.to_string(), (Value::Builtin(name), false));
    }

    let modules: &[(&str, &[(&str, &str)])] = &[
        ("string", &[
            ("upper", "string.upper"),
            ("lower", "string.lower"),
            ("sub", "string.sub"),
            ("find", "string.find"),
            ("split", "string.split"),
            ("join", "string.join"),
            ("rep", "string.rep"),
            ("trim", "string.trim"),
            ("starts_with", "string.starts_with"),
            ("ends_with", "string.ends_with"),
            ("chars", "string.chars"),
            ("from_tuple", "string.from_tuple"),
        ]),
        ("math", &[
            ("abs", "math.abs"),
            ("floor", "math.floor"),
            ("ceil", "math.ceil"),
            ("round", "math.round"),
            ("sqrt", "math.sqrt"),
            ("min", "math.min"),
            ("max", "math.max"),
            ("random", "math.random"),
            ("randomseed", "math.randomseed"),
        ]),
        ("iter", &[
            ("map", "iter.map"),
            ("filter", "iter.filter"),
            ("reduce", "iter.reduce"),
        ]),
        ("struct", &[
            ("keys", "struct.keys"),
            ("values", "struct.values"),
            ("entries", "struct.entries"),
            ("has", "struct.has"),
            ("merge", "struct.merge"),
            ("remove", "struct.remove"),
            ("from_tuple", "struct.from_tuple"),
            ("from_pairs", "struct.from_pairs"),
            ("zip", "struct.zip"),
        ]),
        ("tuple", &[
            ("get", "tuple.get"),
            ("slice", "tuple.slice"),
            ("concat", "tuple.concat"),
            ("push", "tuple.push"),
            ("reverse", "tuple.reverse"),
            ("index_of", "tuple.index_of"),
            ("contains", "tuple.contains"),
            ("sort", "tuple.sort"),
            ("zip", "tuple.zip"),
        ]),
        ("file", &[
            ("read", "file.read"),
            ("write", "file.write"),
            ("append", "file.append"),
            ("exists", "file.exists"),
        ]),
        ("datetime", &[
            ("now", "datetime.now"),
            ("format", "datetime.format"),
            ("timestamp", "datetime.timestamp"),
            ("from_timestamp", "datetime.from_timestamp"),
        ]),
    ];

    for (mod_name, fns) in modules {
        let mut fields = StructMap::new();
        for (name, full) in *fns {
            fields.insert(OrderedKey(name.to_string()), Value::Builtin(full));
        }
        let s = Value::Struct(Rc::new(RefCell::new(fields)));
        globals.insert(mod_name.to_string(), (s, false));
    }
}
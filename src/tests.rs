
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct TestSimple {
    a: i32,
    b: bool,
    c: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct TestComplex {
    a: Vec<TestSimple>,
    b: BTreeMap<String, TestComplex>,
}

use super::*;

fn fixture_data_simple() -> TestSimple {
    TestSimple {
        a: 123,
        b: true,
        c: "test".to_owned(),
    }
}

fn fixture_data_complex(req: u32) -> TestComplex {
    TestComplex {
        a: vec![
            fixture_data_simple(),
            fixture_data_simple(),
            fixture_data_simple(),
        ],
        b: if req == 0 {
            BTreeMap::new()
        } else {
            BTreeMap::from([("test".to_owned(), fixture_data_complex(req - 1))])
        },
    }
}

#[test]
fn test_map_access_with_names() {
    let data = BuilderDataType::Map(vec![
        (
            BuilderDataType::String(Cow::from("a")),
            BuilderDataType::Integer(123),
        ),
        (
            BuilderDataType::String(Cow::from("b")),
            BuilderDataType::Boolean(true),
        ),
        (
            BuilderDataType::String(Cow::from("c")),
            BuilderDataType::String(Cow::from("test")),
        ),
    ]);

    let test: TestSimple = from_data(data).unwrap();

    assert_eq!(fixture_data_simple(), test);
}
#[test]
fn test_map_access_with_idnex() {
    let data = BuilderDataType::Map(vec![
        (BuilderDataType::Unsigned(0), BuilderDataType::Integer(123)),
        (BuilderDataType::Unsigned(1), BuilderDataType::Boolean(true)),
        (
            BuilderDataType::Unsigned(2),
            BuilderDataType::String(Cow::from("test")),
        ),
    ]);

    let test: TestSimple = from_data(data).unwrap();

    assert_eq!(fixture_data_simple(), test);
}
#[test]
fn test_list_access() {
    let data = BuilderDataType::List(vec![
        BuilderDataType::Integer(123),
        BuilderDataType::Boolean(true),
        BuilderDataType::String(Cow::from("test")),
    ]);

    let test: TestSimple = from_data(data).unwrap();

    assert_eq!(fixture_data_simple(), test);
}

#[test]
fn test_complex() {
    let data = BuilderDataType::Reference(Rc::new(BuilderDataType::List(vec![
        BuilderDataType::Integer(123),
        BuilderDataType::Boolean(true),
        BuilderDataType::String(Cow::from("test")),
    ])));

    let data = BuilderDataType::Reference(Rc::new(BuilderDataType::Repeat(vec![
        BuilderDataType::Unsigned(3),
        data,
    ])));

    let data = BuilderDataType::List(vec![data, BuilderDataType::Map(vec![])]);

    let test: TestComplex = from_data(data).unwrap();

    assert_eq!(fixture_data_complex(0), test);
}

#[test]
fn test_complex_nested() {
    let data = BuilderDataType::Reference(Rc::new(BuilderDataType::List(vec![
        BuilderDataType::Integer(123),
        BuilderDataType::Boolean(true),
        BuilderDataType::String(Cow::from("test")),
    ])));

    let data = BuilderDataType::Reference(Rc::new(BuilderDataType::Repeat(vec![
        BuilderDataType::Unsigned(3),
        data,
    ])));

    let nest_count = Rc::new(RefCell::new(BuilderDataType::Integer(3)));

    let data = Rc::new_cyclic(|self_reference| {
        BuilderDataType::List(vec![
            data,
            BuilderDataType::IfThenElse(vec![
                BuilderDataType::Take(nest_count),
                BuilderDataType::Map(vec![(
                    BuilderDataType::String(Cow::from("test")),
                    BuilderDataType::SelfReference(self_reference.clone()),
                )]),
                BuilderDataType::Map(vec![]),
            ]),
        ])
    });

    let test: TestComplex = from_ref(&data).unwrap();

    assert_eq!(fixture_data_complex(3), test);
}

#[test]
fn test_closure() {
    let data = BuilderDataType::Closure(vec![
        BuilderDataType::List(vec![
            BuilderDataType::Argument(1),
            BuilderDataType::IfThenElse(vec![
                BuilderDataType::TakeFromArgument(2),
                BuilderDataType::Map(vec![(
                    BuilderDataType::String(Cow::from("test")),
                    BuilderDataType::Argument(0),
                )]),
                BuilderDataType::Map(vec![]),
            ]),
        ]),
        BuilderDataType::Repeat(vec![
            BuilderDataType::Unsigned(3),
            BuilderDataType::List(vec![
                BuilderDataType::Integer(123),
                BuilderDataType::Boolean(true),
                BuilderDataType::String(Cow::from("test")),
            ]),
        ]),
        BuilderDataType::Integer(3),
    ]);

    let test: TestComplex = from_data(data).unwrap();

    assert_eq!(fixture_data_complex(3), test);
}
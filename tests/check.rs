// tests/layout_tests.rs

// Import the macro from your crate (assume your crate is named `soaaos`).
use soaaos::layout;
use std::error::Error;

//
// Test for the Struct-of-Arrays (SOA) layout.
//
#[layout("struct-of-arrays")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SoaTest {
    pub field1: u32,
    pub field2: String,
}

#[test]
fn soa_add_and_getters() {
    // The generated layout type is named `<StructName>sLayout`,
    // so here we get `SoaTestsLayout`.
    let mut layout = SoaTestsLayout::new();
    assert!(layout.is_empty());

    // Add an item to the layout.
    let item = SoaTest {
        field1: 42,
        field2: "hello".into(),
    };
    let id = layout.add(item);
    assert_eq!(layout.len(), 1);

    // Use the generated getters (e.g. `get_field1`, `get_field2`).
    assert_eq!(layout.get_field1(id).unwrap(), &42);
    assert_eq!(layout.get_field2(id).unwrap(), "hello");
}

#[test]
fn soa_iterators() {
    let mut layout = SoaTestsLayout::new();
    layout.add(SoaTest {
        field1: 1,
        field2: "a".to_string(),
    });
    layout.add(SoaTest {
        field1: 2,
        field2: "b".to_string(),
    });

    // Test the iterator returning a reference view into each element.
    let mut iter = layout.iter();
    let first = iter.next().unwrap();
    assert_eq!(first.field1, &1);
    assert_eq!(first.field2, "a");

    let second = iter.next().unwrap();
    assert_eq!(second.field1, &2);
    assert_eq!(second.field2, "b");
    assert!(iter.next().is_none());
}

//
// Test for the Array-of-Structs (AOS) layout.
//
#[layout("array-of-structs")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AosTest {
    pub value: i32,
    pub text: String,
}

#[test]
fn aos_add_and_getters() {
    // The generated layout type is named `AosTestsLayout`.
    let mut layout = AosTestsLayout::new();
    let item = AosTest {
        value: 10,
        text: "world".to_string(),
    };
    let id = layout.add(item);
    assert_eq!(layout.len(), 1);

    // Check the generated getters.
    assert_eq!(layout.get_value(id).unwrap(), &10);
    assert_eq!(layout.get_text(id).unwrap(), "world");
}

#[test]
fn aos_iterators() {
    let mut layout = AosTestsLayout::new();
    layout.add(AosTest {
        value: 3,
        text: "foo".to_string(),
    });
    layout.add(AosTest {
        value: 4,
        text: "bar".to_string(),
    });

    let mut iter = layout.iter();
    let first = iter.next().unwrap();
    assert_eq!(first.value, &3);
    assert_eq!(first.text, "foo");

    let second = iter.next().unwrap();
    assert_eq!(second.value, &4);
    assert_eq!(second.text, "bar");
    assert!(iter.next().is_none());
}

//
// Test the diff method on the SOA layout.
// (A similar test could be written for the AOS layout.)
//
#[test]
fn diff_method() {
    let mut layout1 = SoaTestsLayout::new();
    let mut layout2 = SoaTestsLayout::new();

    layout1.add(SoaTest {
        field1: 100,
        field2: "diff".to_string(),
    });
    layout2.add(SoaTest {
        field1: 100,
        field2: "diff".to_string(),
    });
    // When both layouts are identical, diff should return None.
    assert!(layout1.diff(&layout2).is_none());

    // Add a second element that differs in one field.
    layout1.add(SoaTest {
        field1: 200,
        field2: "same".to_string(),
    });
    layout2.add(SoaTest {
        field1: 200,
        field2: "changed".to_string(),
    });

    let diff = layout1.diff(&layout2);
    assert!(diff.is_some());
    let diff_str = diff.unwrap();
    // Expect the diff output to mention the differing field (e.g., "field2").
    assert!(diff_str.contains("field2"));
}

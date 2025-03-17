use soaaos::layout;
use std::error::Error;

#[test]
fn test_soa() {
    #[layout("soa")]
    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
    struct NodeSoa {
        op: u8,
        arg1: u16,
        arg2: u16,
    }

    #[layout("aos")]
    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
    struct NodeAos {
        op: u8,
        arg1: u16,
        arg2: u16,
    }

    let mut nodes_soa = NodeSoasLayout::new();
    let mut nodes_aos = NodeAossLayout::new();
    for i in 0..3 {
        nodes_soa.add(NodeSoa {
            op: u8::from(i),
            arg1: u16::from(i * 10),
            arg2: u16::from(i * 20),
        });
        nodes_aos.add(NodeAos {
            op: u8::from(i),
            arg1: u16::from(i * 10),
            arg2: u16::from(i * 20),
        });
    }

    assert_eq!(
        nodes_soa.op().collect::<Vec<_>>(),
        nodes_aos.op().collect::<Vec<_>>()
    );
    assert_eq!(
        nodes_soa.arg1().collect::<Vec<_>>(),
        nodes_aos.arg1().collect::<Vec<_>>()
    );
    assert_eq!(
        nodes_soa.arg2().collect::<Vec<_>>(),
        nodes_aos.arg2().collect::<Vec<_>>()
    );

    insta::assert_debug_snapshot!(nodes_soa);
    insta::assert_debug_snapshot!(nodes_aos);
}

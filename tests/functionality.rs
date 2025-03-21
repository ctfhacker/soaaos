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

#[test]
fn test_diff_soa() {
    #[layout("soa")]
    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
    struct NodeSoa {
        op: u8,
        arg1: u16,
        arg2: u16,
    }

    let mut nodes_soa = NodeSoasLayout::new();
    let mut nodes_soa2 = NodeSoasLayout::new();
    for i in 0..3_u8 {
        let mut node = NodeSoa {
            op: u8::from(i),
            arg1: u16::from(i * 10),
            arg2: u16::from(i * 20),
        };
        let node2 = node.clone();

        if i == 1 {
            node.arg1 = 31337;
        }

        nodes_soa.add(node);
        nodes_soa2.add(node2);
    }

    let Some(diff) = nodes_soa.diff(&nodes_soa2) else {
        panic!("Expected diff not found");
    };

    insta::assert_snapshot!(diff);
}

#[test]
fn test_diff_aos() {
    #[layout("aos")]
    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
    struct NodeAos {
        op: u8,
        arg1: u16,
        arg2: u16,
    }

    let mut nodes_soa = NodeAossLayout::new();
    let mut nodes_soa2 = NodeAossLayout::new();
    for i in 0..3_u8 {
        let mut node = NodeAos {
            op: u8::from(i),
            arg1: u16::from(i * 10),
            arg2: u16::from(i * 20),
        };
        let node2 = node.clone();

        if i == 1 {
            node.arg1 = 31337;
        }

        nodes_soa.add(node);
        nodes_soa2.add(node2);
    }

    let Some(diff) = nodes_soa.diff(&nodes_soa2) else {
        panic!("Expected diff not found");
    };

    insta::assert_snapshot!(diff);
}

use rand::distr::SampleString;
use rand::prelude::SliceRandom;
use rand::{Rng, RngCore};

use soaaos::layout;
use std::error::Error;
use std::time::{Duration, Instant};

trait ArchReg: PartialEq + std::fmt::Debug {
    fn name(self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Register {
    A,
    B,
    C,
}

impl ArchReg for Register {
    fn name(self) -> &'static str {
        match self {
            Register::A => "A",
            Register::B => "B",
            Register::C => "C",
        }
    }
}

#[layout("soa")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node<R>
where
    R: ArchReg,
{
    op: u8,
    arg: Option<R>,
}

/*
#[layout("aos")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entity {
    op: u8,
    name: String,
    address: u64,
    connections: [usize; 4],
}

impl Entity {
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let len = rng.next_u32() % 1024 + 100;

        let name = rand::distr::Alphanumeric.sample_string(rng, len as usize);
        let connections = [
            rng.next_u64() as usize,
            rng.next_u64() as usize,
            rng.next_u64() as usize,
            rng.next_u64() as usize,
        ];

        Self {
            op: rng.next_u32() as u8,
            name,
            address: rng.next_u64(),
            connections,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Collection {
    StructOfArrays,
    HashMap,
    BTreeMap,
}

#[inline(never)]
pub fn find_val<'a>(soa: &'a EntitysLayout, val: &'a str) -> Option<&'a u64> {
    let mut count = 0;
    for curr_val in soa.name() {
        if curr_val == val {
            return soa.get_address(EntityId(count as u32)).ok();
        }
        count += 1;
    }

    None
}

fn test() {
    for size in [4, 10, 100, 1000, 10000, 100000] {
        let mut rng = rand::rng();

        let mut oracle = Vec::new();

        let mut soa = EntitysLayout::with_capacity(size);
        let mut hashmap = std::collections::HashMap::with_capacity(size);
        let mut btreemap = std::collections::BTreeMap::new();

        for _ in 0..size {
            oracle.push(Entity::random(&mut rng));
        }

        // Create the collections for this test
        for entity in oracle.iter() {
            soa.add(entity.clone());
            hashmap.insert(entity.name.clone(), entity.clone());
            btreemap.insert(entity.name.clone(), entity.clone());
        }

        let mut collections = [
            Collection::HashMap,
            Collection::BTreeMap,
            Collection::StructOfArrays,
        ];

        collections.shuffle(&mut rng);

        let mut times = [
            Duration::from_secs(0),
            Duration::from_secs(0),
            Duration::from_secs(0),
        ];

        for _ in 0..size {
            for collection in collections {
                let index = rng.next_u64() as usize % size;
                let entity = *oracle.get(index).as_ref().unwrap();

                match collection {
                    Collection::StructOfArrays => {
                        let start = Instant::now();
                        let res = find_val(&soa, &entity.name).copied();
                        let elapsed = start.elapsed();

                        assert_eq!(res, Some(entity.address));
                        times[collection as usize] += elapsed;
                    }
                    Collection::HashMap => {
                        let start = Instant::now();
                        let res = hashmap.get(&entity.name);
                        let elapsed = start.elapsed();

                        assert_eq!(res.map(|e| e.address), Some(entity.address));
                        times[collection as usize] += elapsed;
                    }
                    Collection::BTreeMap => {
                        let start = Instant::now();
                        let res = btreemap.get(&entity.name);
                        let elapsed = start.elapsed();

                        assert_eq!(res.map(|e| e.address), Some(entity.address));
                        times[collection as usize] += elapsed;
                    }
                }
            }
        }

        println!("-- SIZE {size} --");
        println!("SOA:      {:?}", times[Collection::StructOfArrays as usize]);
        println!("HashMap:  {:?}", times[Collection::HashMap as usize]);
        println!("BTreeMap: {:?}", times[Collection::BTreeMap as usize]);
    }
}
*/

fn main() {
    let mut nodes = NodesLayout::<Register>::new();

    nodes.add(Node {
        op: 8,
        arg: Some(Register::B),
    });
    nodes.add(Node {
        op: 1,
        arg: Some(Register::A),
    });
    nodes.add(Node {
        op: 2,
        arg: Some(Register::C),
    });

    dbg!(&nodes);

    for n in nodes.iter_enumerated() {
        dbg!(&n);
    }

    // test();
}

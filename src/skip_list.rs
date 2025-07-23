//- skip_list
//- --
//- An implementation of the Skip List data structure in Rust.
//- By [masonblier](https://github.com/masonblier) 2025-07-07

//- Skip lists are a sorted, fast-access data structure with similar performance
//- characteristics to a balanced binary tree, made famous in the
//- [MIT OpenCourseWare lecture by Professor Srinivas Devadas](https://www.youtube.com/watch?v=2g9OSRKJuzM).
//-
//- Having `O(log(N))` insertion and retrieval, the main advantage skip lists have over a binary
//- tree is that they never have to be rebalanced, the data in memory stays largely consistent over
//- many write operations. However, as data structure with randomization as a core feature, the
//- performance can vary depending on the outcome of the random number generator, which determines
//- which items are propagated to the "fast lane" search vectors. They also use more memory compared
//- to a compactified binary tree. Some databases using Skip Lists in production include
//- [Apache Lucene](https://lucene.apache.org/), [Redis](https://redis.io/), and [RocksDB](https://rocksdb.org/).
//=

//- Type Definitions
//- ==
//=
//- Starting off, I import some useful tools. The PartialOrd trait defines comparator functions for
//- types, which is required so I sort the values inserted into the SkipList for fast access.
//- fmt is used later on to define a convenient display function, showing the inner structure of the list.
//- And critically, rand::Rng is a third-party library for generating random numbers, used to probabilistically
//- distribute the values in the fast-lane.
use std::cmp::PartialOrd;
use std::fmt;
use rand::Rng;

//- The number of "fast-lane" layers affects the performance traits of the list. A single layer is
//- equivalent to a flat linked-list, with no performance gains. But too many layers increases the
//- memory usage with duplicated data. The ideal number of layers is the `log2(N)` of the number of elements
//- in the list, but here I chose a smaller number to make debugging simpler.
const SKIP_LIST_LAYERS: usize = 3;

//- The `SkipList` struct is the public-facing type which users can interact with. It defines a number
//- of public methods below to allow inserting and retrieving values, but the internal data is private.
//- The layers field holds all the nodes in an unsorted `Vec`. The order does not matter, as ordering is done
//- by following links between the nodes. I chose to store these nodes in a `Vec`, with links referring to
//- indexes in the `Vec`, to avoid complications with the borrow checker involving Rust types with duplicate
//- references to the same data.

//- The rust book [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/index.html)
//- is an entertaining tale of one developer's journey in implementing a Linked List in Rust, encountering and
//- detailing the many complex situations with accessing and mutating shared references.

//- Here, I avoid all of this by using `usize` indexing into a `Vec`, similar to how an arena would handle memory.
//- The tradeoff is an additional layer of indirection, and the requirement that these indexes cannot
//- change or move around. But it is not 'unsafe', as to mutate any of the internal data, a proper mutable
//- reference to the `SkipList` is still required.
pub struct SkipList<T> {
    layers: [Vec<SkipListNode<T>>; SKIP_LIST_LAYERS],
    head: [Option<usize>; SKIP_LIST_LAYERS],
}

//- The SkipListNode is an internal type storing links between values, and links from fast-lane values
//- down to lower layers. Only the lowest layer in the SkipList, `layers[0]`, stores all values. Nodes in
//- the lowest layer have no `down` references.
struct SkipListNode<T> {
    value: T,
    next: Option<usize>,
    down: Option<usize>,
}

//- Implementation
//- ==
//=
//- Here are implementations of the public-facing functions for this type.

//- Implementing `Default` as the constructor allows this struct to be used automatically in any other types
//- which implement the `Default` trait, including types defined with the `#[derive(Default)]` attribute.
//- The initial data is a list of empty `Vec`s for each layer of the data structure, and a set of empty head
//- references which will eventually point to the first node in each layer.
impl<T> Default for SkipList<T> {
    fn default() -> Self {
        Self {
            layers: [const { Vec::new() }; SKIP_LIST_LAYERS],
            head: [const { None }; SKIP_LIST_LAYERS],
        }
    }
}

//- While `SkipList` is generic, and can take user-provided types of `T`, it requires that `T` implements
//- the `PartialOrd` and `Clone` traits. `PartialOrd` is required for `<`, `>`, and `==` comparisons critical
//- for any sorted data structure. `Clone` is required so that copies of the values can be stored directly in
//- the fast-lanes, allowing for faster searching. For large values, it would be better to store references
//- to `&T` to reduce memory duplication, but it would require managing shared references to ensure compliance
//- with the borrow checker.
impl<T: PartialOrd + Clone> SkipList<T> {

//- The `insert` method is where all the fun happens in a `SkipList`. The new value must be inserted in a sorted
//- location in the list, and based on the roll of a random number generator, inserted into a number of fast-lanes
//- to speed up searching for the value. The more fast-lanes a value is in, the faster it can be found, but if too
//- many values are inserted into the fast lanes, the whole list slows down, up to a worst-case amoratized scenario
//- of linear time.
    pub fn insert(&mut self, v: T) {
        // randomize number of insert layers
        let num_insert_layers = if self.head[0].is_none() {
            // insert node into all layers
            SKIP_LIST_LAYERS
        } else {
            1 + rand::rng().random_range(0..SKIP_LIST_LAYERS)
        };

//- Finding the insertion point requires going through the layers, first from the most-sparse layer, then down to
//- the most complete layer, until the insert location of the value is found in all layers. A list of the nodes
//- closest to the insert point is kept to quickly update these nodes with pointers to the newly inserted node.
        let mut layer_start_idx = self.head[SKIP_LIST_LAYERS-1];
        // store list of node idxs for insertion
        let mut insert_list = [const { None }; SKIP_LIST_LAYERS];
        // for each layer
        for rlayer in 0..SKIP_LIST_LAYERS {
            let layer = SKIP_LIST_LAYERS - 1 - rlayer;
            let mut node_idx = layer_start_idx.clone();
            // if front insertion
            if node_idx.is_none() || v < self.layers[layer][node_idx.unwrap()].value  {
                insert_list[layer] = None;
                // set next layer start idx to none
                if layer > 0 {
                    layer_start_idx = self.head[layer-1];
                }

            // if existing element
            } else if v == self.layers[layer][node_idx.unwrap()].value {
                return;

            // else, find insertion index of current layer
            } else {
                loop {
                    // check the next node in the sequence
                    let next_idx = self.layers[layer][node_idx.unwrap()].next;
                    // if the end of the sequence has been reached, use last node
                    if next_idx.is_none() {
                        break;
                    }
                    // if the next node is greater than the value, use last node
                    if v < self.layers[layer][next_idx.unwrap()].value {
                        break;
                    }
                    // continue
                    node_idx = next_idx;
                }

                // set insert index to found node
                insert_list[layer] = node_idx.clone();
                // set next layer start idx to down value of found node
                layer_start_idx = self.layers[layer][node_idx.unwrap()].down;
            }
        }

//- Now that I have a list of nodes which need to be updated, the new node can be inserted. Starting with
//- the lowest layer this time, the node is inserted into the unsorted `Vec` of all nodes, and the index to
//- the new node is set as the `next` of the node it was inserted after.

//- I continue up the layers for the amount of layers given by the random number generator. For each layer,
//- the new node is inserted, the neighboring links are updated, and the `down` is set to the index of the
//- node in the lower layer.
        let mut last_insert: Option<usize> = None;
        for layer in 0..num_insert_layers {
            // get old next value of insert-parent node
            let next = if let Some(insert_idx) = insert_list[layer] {
                self.layers[layer][insert_idx].next.clone()
            } else {
                self.head[layer]
            };
            // insert new node into memory
            self.layers[layer].push(
                SkipListNode {
                    value: v.clone(),
                    next,
                    down: last_insert.clone(),
                }
            );
            last_insert = Some(self.layers[layer].len() - 1);
            // update next value of insert-parent node
            if let Some(insert_idx) = insert_list[layer] {
                self.layers[layer][insert_idx].next = last_insert.clone();
            } else {
                self.head[layer] = last_insert.clone();
            }
        }
    }

//- The `contains` method is similar to the first part of the `insert` method, I trace from the sparsest
//- layer to find the nearest node to the value, which directs `down` to the point in the lower layer where
//- the next search can begin. This continues until the value is found, at which point we know the list contains
//- the value and `true` is returned, or until the node is confirmed missing and `false` can be returned.
    pub fn contains(&self, v: T) -> bool {
        // starting point of layer
        let mut layer_start_idx = self.head[SKIP_LIST_LAYERS-1];
        // for each layer
        for rlayer in 0..SKIP_LIST_LAYERS {
            let layer = SKIP_LIST_LAYERS - 1 - rlayer;
            let mut node_idx = layer_start_idx.clone();
            // if empty
            if node_idx.is_none() {
                return false;
            }
            // if immediate match
            if v == self.layers[layer][node_idx.unwrap()].value {
                return true;
            }
            // keep looking
            loop {
                let next_idx = self.layers[layer][node_idx.unwrap()].next;
                // end of list
                if next_idx.is_none() {
                    break;
                }
                // passed value
                if v < self.layers[layer][next_idx.unwrap()].value {
                    break;
                }
                // continue
                node_idx = next_idx;
            }
            // set next layer start idx to down value of last closest node
            layer_start_idx = self.layers[layer][node_idx.unwrap()].down;
        }
        // no match
        false
    }
}

//- This `fmt` implementation is used for Rust's `Display` trait, allowing the entire list to be
//- formatted as a string for printing to the console. This method gives a view of each layer,
//- showing the tree-like structure of the `SkipList`. An example of the output:
//- ```
//- SkipList
//- [      03,  07,                      18,  22,      ]
//- [ 01,  03,  07,       11,  12,  16,  18,  22,  99, ]
//- [ 01,  03,  07,  09,  11,  12,  16,  18,  22,  99, ]
//- ```
impl<T: PartialOrd + Clone + fmt::Display> fmt::Display for SkipList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut base_list = Vec::<T>::new();
        let mut node_idx = self.head[0];
        while node_idx.is_some() {
            let node = &self.layers[0][node_idx.unwrap()];
            base_list.push(node.value.clone());
            node_idx = node.next;
        }
        let out1 = (1..SKIP_LIST_LAYERS).map(|rlayer| {
            let layer = SKIP_LIST_LAYERS - rlayer;
            let mut outln = "[".to_string();
            let mut node_idx = self.head[layer];
            for b in &base_list {
                if node_idx.is_some() {
                    let node = &self.layers[layer][node_idx.unwrap()];
                    if *b == node.value {
                        outln += format!(" {:02}, ", node.value).as_str();
                        node_idx = node.next;
                    } else {
                        outln += "     ";
                    }
                } else {
                    outln += "     ";
                }
            }
            outln += "]";
            outln
        }).collect::<Vec<String>>().join("\n");
        let out2 = base_list.iter()
            .map(|v| format!(" {:02}, ", v)).collect::<Vec<String>>().join("");
        let out = out1 + "\n[" + &out2 + "]";
        write!(f, "{}", out)
    }
}

//- Included unit-tests can be run from `cargo test` for a quick smoke-test verification
//- of the public methods.
//- ```
//- running 1 test
//- test skip_list::test::basics ... ok
//-
//- test result: ok. 1 passed; 0 failed; 0 ignored;
//-   0 measured; 0 filtered out; finished in 0.00s
//- ```
#[cfg(test)]
mod test {
    use super::SkipList;

    #[test]
    fn basics() {
        let nums: [i64; 10] = [3, 1, 9, 12, 11, 16, 99, 18, 7, 22];

        let mut skip_list = SkipList::default();
        for n in &nums {
            skip_list.insert(*n);
        }

        assert_eq!(skip_list.contains(3), true);
        assert_eq!(skip_list.contains(4), false);
    }
}

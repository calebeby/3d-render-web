use std::{cmp::Ordering, collections::BinaryHeap};

use crate::bijection::Bijection;

#[derive(Debug)]
struct TrieNode<T> {
    children: Vec<(usize, TrieNode<T>)>,
    data: Option<T>,
}

#[derive(Debug)]
pub struct BijectionTrie<T> {
    root: TrieNode<T>,
}

impl<T: std::fmt::Debug> BijectionTrie<T> {
    fn new() -> BijectionTrie<T> {
        BijectionTrie {
            root: TrieNode {
                children: vec![],
                data: None,
            },
        }
    }
    fn insert(&mut self, bijection: &Bijection, data: T) {
        let mut node = &mut self.root;
        for mapping in &bijection.0 {
            let found_child_index = node
                .children
                .iter_mut()
                .position(|(child_mapping, _)| child_mapping == mapping);
            if let Some(found_child_index) = found_child_index {
                node = &mut node.children[found_child_index].1;
            } else {
                let new_child = TrieNode {
                    children: vec![],
                    data: None,
                };
                node.children.push((*mapping, new_child));
                node = &mut node.children.last_mut().unwrap().1;
            }
        }
        node.data = Some(data);
    }
    fn find_exact_bijection(&self, bijection: &Bijection) -> Option<&T> {
        let mut node = &self.root;
        for mapping in &bijection.0 {
            let found_child = node
                .children
                .iter()
                .find(|(child_mapping, _)| child_mapping == mapping);
            if let Some((_, found_child)) = found_child {
                node = found_child;
            } else {
                return None;
            }
        }
        node.data.as_ref()
    }
    fn find_most_similar(&self, search_bijection: &Bijection) -> impl Iterator<Item = (usize, &T)> {
        #[derive(Debug)]
        struct HeapItem<'a, T> {
            node: &'a TrieNode<T>,
            differences: usize,
            index: usize,
        }

        impl<'a, T> Ord for HeapItem<'a, T> {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse because we want to see minimum-difference items first
                self.differences.cmp(&other.differences).reverse()
            }
        }

        impl<'a, T> PartialOrd for HeapItem<'a, T> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<'a, T> PartialEq for HeapItem<'a, T> {
            fn eq(&self, other: &Self) -> bool {
                self.differences == other.differences
            }
        }

        impl<'a, T> Eq for HeapItem<'a, T> {}

        struct SimilarityIterator<'a, T> {
            heap: BinaryHeap<HeapItem<'a, T>>,
            search_bijection: Bijection,
        }
        impl<'a, T: std::fmt::Debug> Iterator for SimilarityIterator<'a, T> {
            type Item = (usize, &'a T);
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let next_heap_item = self.heap.pop()?;
                    for (value, child) in &next_heap_item.node.children {
                        self.heap.push(HeapItem {
                            node: child,
                            differences: if *value == self.search_bijection.0[next_heap_item.index]
                            {
                                next_heap_item.differences
                            } else {
                                next_heap_item.differences + 1
                            },
                            index: next_heap_item.index + 1,
                        })
                    }
                    if let Some(data) = &next_heap_item.node.data {
                        return Some((next_heap_item.differences, data));
                    }
                }
            }
        }

        let mut heap = BinaryHeap::new();

        heap.push(HeapItem {
            node: &self.root,
            differences: 0,
            index: 0,
        });
        SimilarityIterator {
            search_bijection: search_bijection.clone(),
            heap,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_exact() {
        let mut trie = BijectionTrie::new();
        trie.insert(&Bijection(vec![1, 2, 3]), "123");
        trie.insert(&Bijection(vec![1, 2, 4]), "124");
        trie.insert(&Bijection(vec![2, 9, 1]), "291");

        assert_eq!(
            trie.find_exact_bijection(&Bijection(vec![1, 2, 3])),
            Some(&"123")
        );
        assert_eq!(
            trie.find_exact_bijection(&Bijection(vec![1, 2, 4])),
            Some(&"124")
        );
        assert_eq!(
            trie.find_exact_bijection(&Bijection(vec![2, 9, 1])),
            Some(&"291")
        );
        assert_eq!(trie.find_exact_bijection(&Bijection(vec![1, 2, 5])), None);
    }
    #[test]
    fn test_find_most_similar() {
        let mut trie = BijectionTrie::new();
        trie.insert(&Bijection(vec![1, 2, 3]), "123");
        trie.insert(&Bijection(vec![1, 2, 4]), "124");
        trie.insert(&Bijection(vec![2, 9, 1]), "291");
        trie.insert(&Bijection(vec![2, 2, 3]), "223");
        trie.insert(&Bijection(vec![2, 2, 4]), "224");

        assert_eq!(
            trie.find_most_similar(&Bijection(vec![1, 2, 3]))
                .collect::<Vec<_>>(),
            vec![
                (0, &"123"),
                (1, &"124"),
                (1, &"223"),
                (2, &"224"),
                (3, &"291")
            ],
        );

        assert_eq!(
            trie.find_most_similar(&Bijection(vec![1, 2, 4]))
                .collect::<Vec<_>>(),
            vec![
                (0, &"124"),
                (1, &"123"),
                (1, &"224"),
                (2, &"223"),
                (3, &"291")
            ],
        );

        // 5's do not appear
        assert_eq!(
            trie.find_most_similar(&Bijection(vec![5, 5, 5]))
                .collect::<Vec<_>>(),
            vec![
                (3, &"123"),
                (3, &"291"),
                (3, &"224"),
                (3, &"124"),
                (3, &"223")
            ],
        );
    }
}

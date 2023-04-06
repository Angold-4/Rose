// src/btree.rs

/*
* B Tree Implementation
* In the current implementation, both internal and leaf nodes store key-value pairs
*
* The properties of a B-tree are:
* 1. Every node has at most 2 * B - 1 keys
* 2. All keys in a node are in the ascending order
* 3. All keys in the subtree rooted at a child node `i` are greater than the key at index `i - 1`
*    and less than the key at index `i`
*
* ############################################################################################
*
* For example, consider this internal node:
*
* keys: [K1, K2, K3]
* values: [V1, V2, V3]
* children: [C0, C1, C2, C3]
*
* Here, all the keys in the subtree rooted at C0 are less than K1, all the keys in the subtree
* rooted at C1 are between K1 and K2, etc.
*
*
* the B-Tree maintains its properties by carefully inserting and splitting nodes
* The insert_non_full function ensures that keys are inserted in the correct order.
* The split_child function takes care of splitting nodes when they become full.
*
* ############################################################################################
*
* For the split_child function, maybe it is pretty hard to understand, let's draw some figures:
* Let's say B is 2, here is the current B-Tree:
*     [5]
*    /   \
* {2, 4} {6, 8, 9}
*
* Now, let's say we want to insert the kv pair (7, V7). The root node is not full,
* so we proceed to insert the key-value pair into the appropriate child node. In this case,
* it is the right child node, which is already full:
*
*     [5]
*    /   \
* {2, 4} {6, 8, 9, 7}
*
* Since the right child node is full, we need to split it, the split_child fn will be called:
*
* 1. Identify the middle key and value (8, V8) in this case.
* 2. Create a new node to store the keys and values to the right of the middle key. (9, V9)
* 3. Remove the keys and values to the right of the middle key from the original node, as well as
*    the middle key and value.
* 4. Insert the middle key and value into the parent node (root) at the appropriate position.
* 5. Add the newly created node as a child of the parent node (root) to the right of the orginal
*    child node.
*
* After the split, the tree will look like this:
*    [5, 8]
*    /   |   \
* {2, 4} {6, 7} {9}
*
*
*/

use std::cmp::Ordering;

const B: usize = 3;

pub struct BTree<K: Ord + Clone, V: Clone> {
    root: Option<Box<Node<K, V>>>,
}

#[derive(Clone)]
pub struct Node<K: Ord + Clone, V: Clone> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Box<Node<K, V>>>,
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    pub fn new() -> Self {
        BTree { root: None }
    }

    pub fn insert(&mut self, key: K, value: V) {
        // Insert key-value pair and handle tree updates
        if let Some(root) = &mut self.root { // if root is not None
            // if let patten is checking whether self.root is of type Option<T> and whether it is
            // Some, if it is, then the value inside the Some variant is bound to the var root
            // and the code inside the if let block is executed
            if root.is_full() { // it has 2 * B - 1 keys
                // split it before inserting
                let mut new_root = Box::new(Node::new());
                new_root.children.push(root.clone()); 
                // at least an internal node will have 1 children
                new_root.split_child(0);
                new_root.insert_non_full(key, value);
            } else {
                root.insert_non_full(key, value);
            }
        } else {
            let mut new_root = Box::new(Node::new());
            new_root.keys.push(key);
            new_root.values.push(value);
            self.root = Some(new_root)
            // the Some is just a wrapper, it set the Option of new_root to be Some
        }
    }

    pub fn delete(&mut self, key: &K) {
        // Delete key-value pair and handle tree updates
        if let Some(root) = &mut self.root {
            root.delete(key.clone());
        }
    }

    pub fn search(&self, key: &K) -> Option<&V> {
        // Search for a key and return the associated value if found
        self.root.as_ref().and_then(|root| root.search(key))
    }
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    // Helper methods for B-tree operations (insert, delete, search, etc.)
    // Methods like split, merge, and other utility methods will be implemented here
    fn new() -> Self {
        Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
        }
    }

    fn is_full(&self) -> bool {
        self.keys.len() >= 2 * B - 1
    }

    fn split_child(&mut self, index: usize) {
        // index refers to the child node that needs to be split, self refers to the new_root

        // 1. identify the middle key and value
        let split_key = self.children[index].keys[B - 1].clone();
        let split_value = self.children[index].values[B - 1].clone();

        // 2. Create new node to store the keys and values to right of the middle key
        let mut right = Box::new(Node::new());

        // 3. Remove they keys and values to the right of the middle key from the original node
        // (greater part)
        right.keys = self.children[index].keys.split_off(B); // second half of the keys
        right.values = self.children[index].values.split_off(B);
        // now the self.children[index] becomes the first half of the keys (left)

        if !self.children[index].children.is_empty() {
            // if the original full root has some other childrens, we split the right part of the
            // child into the right part of the new root's child
            // which also means the root is a internal node, will have at least B children
            right.children = self.children[index].children.split_off(B);
        }

        // 4. insert the middle key and value into the root at the appropriate position
        self.keys.insert(index, split_key);
        self.values.insert(index, split_value);

        // 5. Add the newly created node as a child of the parent node to the right of the original
        // child node
        self.children.insert(index + 1, right);
    }

    fn insert_non_full(&mut self, key: K, value: V) {
        let mut index = match self.keys.binary_search(&key) {
            // the reason we are using binary_seach here is to ensure the keys are sorted
            // which means, find the appropriate position for the new key
            Ok(_) => return, // Key already exists, so we don't need to insert it
            Err(index) => index,
        };
        // the index is the new key's position in the self.keys

        /* In a B-Tree, the internal nodes primarily serve as a way to navigate through the tree
        * structure to reach the leaf nodes, where the actual key-value pairs are stored, by
        * always attempting insert the key into a leaf node, we ensure that the tree remains
        * balanced and that the properties of the B-Tree are maintained, the value of internal
        * node will changed only when the current node is full (children), and we need to split it
        */

        if self.children.is_empty() {
            // Leaf node case
            // termination condition (DFS)
            self.keys.insert(index, key);
            self.values.insert(index, value);
        } else {
            // Internal node case
            if self.children[index].is_full() {
                self.split_child(index); // split the current index

                // After splitting, check if the new key should go to the right child
                if self.keys[index].lt(&key) {
                    index += 1;
                }
            }

            self.children[index].insert_non_full(key, value);
        }
    }

    fn search(&self, key: &K) -> Option<&V> {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&self.values[index]),
            Err(index) => {
                if self.children.is_empty() {
                    None
                } else {
                    self.children[index].search(key)
                }
            }
        }
    }

    pub fn delete(&mut self, key: K) {
        match self.keys.binary_search(&key) {
            Ok(index) => {
                // The key is in the current node
                if self.children.is_empty() {
                    // The current node is a leaf node
                    // Then we just simply remove the key and value
                    self.keys.remove(index);
                    self.values.remove(index);
                } else {
                    // The current node is an internal node
                }
            }
            Err(index) => {
            }
        }

    }
}

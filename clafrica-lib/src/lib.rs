//! # Clafrica Lib
//!
//! `clafrica-lib` is a collection of utilities to make handling
//!  of clafrica code more convenient.
//!
//! Example
//! ```
//! use clafrica_lib::{bst, utils};
//!
//! // Build a BST
//! let root = bst::Node::new();
//! root.insert(vec!['a', 'f'], "ɑ".to_owned());
//! root.insert(vec!['a', 'f', '1'], "ɑ̀".to_owned());
//!
//! // Bulk insert of data in the BST
//! let data = vec![["af11", "ɑ̀ɑ̀"], ["?.", "ʔ"]];
//! utils::build_map(data);
//!
//! // or
//! let data = utils::load_data("../tmp/data.txt");
//! let data = data.iter()
//!                .map(|e| [e[0].as_str(), e[1].as_str()])
//!                .collect();
//!
//! utils::build_map(data);
//!
//! // Traverse the tree
//! let node = root.goto('a').and_then(|e| e.goto('f'));
//! assert_eq!(node.unwrap().take(), Some("ɑ".to_owned()));
//! ```

pub mod bst {
    use std::collections::HashMap;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug)]
    pub struct Node<'a> {
        neighbors: RefCell<HashMap<char, Rc<Node<'a>>>>,
        value: RefCell<Option<String>>,
    }

    impl<'a> Default for Node<'a> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<'a> Node<'a> {
        /// Initialize a new node.
        pub fn new() -> Self {
            Self {
                neighbors: HashMap::new().into(),
                value: None.into(),
            }
        }

        /// Insert a path in the BST.
        pub fn insert(&self, path: Vec<char>, value: String) {
            if let Some(character) = path.clone().first() {
                let new_node = Rc::new(Self::new());

                self.neighbors
                    .borrow()
                    .get(character)
                    .unwrap_or(&new_node)
                    .insert(path.into_iter().skip(1).collect(), value);

                self.neighbors
                    .borrow_mut()
                    .entry(*character)
                    .or_insert(new_node);
            } else {
                *self.value.borrow_mut() = Some(value);
            };
        }

        /// Move from one node to another
        pub fn goto(&self, character: char) -> Option<Rc<Self>> {
            self.neighbors.borrow().get(&character).map(Rc::clone)
        }

        /// Extract the value from a node .
        pub fn take(&self) -> Option<String> {
            self.value.borrow().as_ref().map(ToOwned::to_owned)
        }
    }
}

pub mod utils {
    use crate::bst;
    use std::fs;

    /// Load the clafrica code from a plain text file.
    pub fn load_data(file_path: &str) -> Vec<Vec<String>> {
        let data = fs::read_to_string(file_path).unwrap();
        data.split('\n')
            .map(|line| {
                line.split_whitespace()
                    .filter(|token| !token.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .collect()
    }

    /// Build a BST from the clafrica code.
    pub fn build_map(data: Vec<[&str; 2]>) -> bst::Node {
        let root = bst::Node::new();

        data.iter().for_each(|e| {
            root.insert(e[0].chars().collect(), e[1].to_owned());
        });

        root
    }
}

#[cfg(test)]
mod tests {
    fn generate_data_file() {
        use std::fs;
        let data = "af11\t  \t  \t     ɑ̀ɑ̀\naf1\t  \t  \tɑ̀";
        fs::write("../tmp/data.txt", data).unwrap();
    }

    #[test]
    fn test_load_data() {
        use crate::utils;

        generate_data_file();
        utils::load_data("../tmp/data.txt")
            .iter()
            .for_each(|pair| assert_eq!(pair.len(), 2));
    }

    #[test]
    fn test_build_map() {
        use crate::utils;

        let data = vec![["af11", "ɑ̀ɑ̀"], ["?.", "ʔ"]];
        utils::build_map(data);

        generate_data_file();

        let data = utils::load_data("../tmp/data.txt");
        utils::build_map(
            data.iter()
                .map(|e| [e[0].as_str(), e[1].as_str()])
                .collect(),
        );
    }

    #[test]
    fn test_node() {
        use crate::bst;

        let root = bst::Node::new();
        root.insert(vec!['a', 'f'], "ɑ".to_owned());
        root.insert(vec!['a', 'f', '1'], "ɑ̀".to_owned());

        assert!(root.goto('a').is_some());
        assert!(root.goto('b').is_none());

        let node = root.goto('a').and_then(|e| e.goto('f'));
        assert_eq!(node.as_ref().unwrap().take(), Some("ɑ".to_owned()));

        let node = node.and_then(|e| e.goto('1'));
        assert_eq!(node.as_ref().unwrap().take(), Some("ɑ̀".to_owned()));
    }
}

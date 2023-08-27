//! # Clafrica Lib
//!
//! `clafrica-lib` is a collection of utilities to make handling
//!  of clafrica code more convenient.
//!
//! Example
//! ```
//! use clafrica_lib::{text_buffer, utils};
//!
//! // Build a TextBuffer
//! let root = text_buffer::Node::default();
//! root.insert(vec!['a', 'f'], "ɑ".to_owned());
//! root.insert(vec!['a', 'f', '1'], "ɑ̀".to_owned());
//!
//! // Bulk insert of data in the TextBuffer
//! let data = vec![["af11", "ɑ̀ɑ̀"], ["?.", "ʔ"]];
//! utils::build_map(data);
//!
//! // or directly from a file
//! let data = utils::load_data("data/sample.txt")
//!                   .expect("Failed to load the clafrica code file");
//! let data = data.iter()
//!                .map(|e| [e[0].as_str(), e[1].as_str()])
//!                .collect();
//!
//! utils::build_map(data);
//!
//! // Traverse the tree
//! let node = root.goto('a').and_then(|e| e.goto('f'));
//! assert_eq!(node.unwrap().take(), Some("ɑ".to_owned()));
//!
//! // Test our cursor
//! let mut cursor = text_buffer::Cursor::new(root, 10);
//! let code = "af1";
//! code.chars().for_each(|c| {cursor.hit(c);});
//! assert_eq!(cursor.state(), (Some("ɑ̀".to_owned()), 3, '1'));
//! assert_eq!(cursor.undo(), Some("ɑ̀".to_owned()));
//! ```

pub mod text_buffer {
    use std::collections::{HashMap, VecDeque};
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug)]
    pub struct Node {
        neighbors: RefCell<HashMap<char, Rc<Node>>>,
        pub depth: usize,
        pub key: char,
        value: RefCell<Option<String>>,
    }

    impl Default for Node {
        fn default() -> Self {
            Self::new('\0', 0)
        }
    }

    impl Node {
        /// Initialize a new node.
        pub fn new(key: char, depth: usize) -> Self {
            Self {
                neighbors: HashMap::new().into(),
                depth,
                key,
                value: None.into(),
            }
        }

        /// Insert a sequence in the TextBuffer.
        pub fn insert(&self, sequence: Vec<char>, value: String) {
            if let Some(character) = sequence.clone().first() {
                let new_node = Rc::new(Self::new(*character, self.depth + 1));

                self.neighbors
                    .borrow()
                    .get(character)
                    .unwrap_or(&new_node)
                    .insert(sequence.into_iter().skip(1).collect(), value);

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

        /// Return true is the node is at the initial depth
        pub fn is_root(&self) -> bool {
            self.depth == 0
        }
    }

    #[derive(Clone)]
    pub struct Cursor {
        buffer: VecDeque<Rc<Node>>,
        root: Rc<Node>,
    }

    impl Cursor {
        /// Initialize the cursor
        pub fn new(root: Node, capacity: usize) -> Self {
            Self {
                buffer: VecDeque::with_capacity(capacity),
                root: Rc::new(root),
            }
        }
        /// Enter a character and return his corresponding out
        pub fn hit(&mut self, character: char) -> Option<String> {
            let node = self
                .buffer
                .iter()
                .last()
                .unwrap_or(&Rc::new(Node::new('\0', 0)))
                .goto(character)
                .or_else(|| {
                    // We end the current sequence
                    self.insert(Rc::new(Node::new('\0', 0)));
                    // and start a new one
                    self.root.goto(character)
                })
                .unwrap_or(Rc::new(Node::new(character, 0)));

            let out = node.take();
            self.insert(node);

            out
        }

        fn insert(&mut self, node: Rc<Node>) {
            if self.buffer.len() == self.buffer.capacity() {
                self.buffer.pop_front();
            }
            self.buffer.push_back(node);
        }

        /// Remove the previous enter and return his corresponding out
        pub fn undo(&mut self) -> Option<String> {
            let node = self.buffer.pop_back();

            node.and_then(|node| {
                if node.key == '\0' {
                    self.undo()
                } else {
                    node.take()
                }
            })
        }

        /// Return the current state of the cursor
        pub fn state(&self) -> (Option<String>, usize, char) {
            self.buffer
                .iter()
                .last()
                .map(|n| (n.take(), n.depth, n.key))
                .unwrap_or_default()
        }

        /// Return the current sequence in the cursor
        pub fn to_sequence(&self) -> Vec<char> {
            self.buffer.iter().map(|node| node.key).collect()
        }

        /// Clear the memory of the cursor
        pub fn clear(&mut self) {
            self.buffer.clear();
        }
    }
}

pub mod utils {
    use crate::text_buffer;
    use std::{fs, io};

    /// Load the clafrica code from a plain text file.
    pub fn load_data(file_path: &str) -> Result<Vec<Vec<String>>, io::Error> {
        let data = fs::read_to_string(file_path)?;
        let data = data
            .trim()
            .split('\n')
            .map(|line| {
                line.split_whitespace()
                    .filter(|token| !token.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .collect();
        Ok(data)
    }

    /// Build a TextBuffer from the clafrica code.
    pub fn build_map(data: Vec<[&str; 2]>) -> text_buffer::Node {
        let root = text_buffer::Node::default();

        data.iter().for_each(|e| {
            root.insert(e[0].chars().collect(), e[1].to_owned());
        });

        root
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_data() {
        use crate::utils;

        utils::load_data("data/sample.txt")
            .unwrap()
            .iter()
            .for_each(|pair| assert_eq!(pair.len(), 2));
    }

    #[test]
    fn test_build_map() {
        use crate::utils;

        let data = vec![["af11", "ɑ̀ɑ̀"], ["?.", "ʔ"]];
        utils::build_map(data);

        let data = utils::load_data("data/sample.txt").unwrap();
        utils::build_map(
            data.iter()
                .map(|e| [e[0].as_str(), e[1].as_str()])
                .collect(),
        );
    }

    #[test]
    fn test_node() {
        use crate::text_buffer;

        let root = text_buffer::Node::default();

        assert!(root.is_root());

        root.insert(vec!['a', 'f'], "ɑ".to_owned());
        root.insert(vec!['a', 'f', '1'], "ɑ̀".to_owned());

        assert!(root.goto('a').is_some());
        assert!(!root.goto('a').unwrap().is_root());
        assert!(root.goto('b').is_none());

        let node = root.goto('a').and_then(|e| e.goto('f'));
        assert_eq!(node.as_ref().unwrap().take(), Some("ɑ".to_owned()));

        let node = node.and_then(|e| e.goto('1'));
        assert_eq!(node.as_ref().unwrap().take(), Some("ɑ̀".to_owned()));
    }

    #[test]
    fn test_cursor() {
        use crate::text_buffer;
        use crate::utils;

        macro_rules! hit {
            ( $cursor:ident $( $c:expr ),* ) => (
                $( $cursor.hit($c); )*
            );
        }

        macro_rules! undo {
            ( $cursor:ident $occ:expr ) => {
                (0..$occ).into_iter().for_each(|_| {
                    $cursor.undo();
                });
            };
        }

        let data = utils::load_data("data/sample.txt").unwrap();
        let root = utils::build_map(
            data.iter()
                .map(|e| [e[0].as_str(), e[1].as_str()])
                .collect(),
        );

        let mut cursor = text_buffer::Cursor::new(root, 8);

        assert_eq!(cursor.state(), (None, 0, '\0'));

        hit!(cursor '2', 'i', 'a', 'f');
        assert_eq!(cursor.to_sequence(), vec!['\0', '2', 'i', 'a', 'f']);

        assert_eq!(cursor.state(), (Some("íɑ́".to_owned()), 4, 'f'));

        undo!(cursor 1);
        assert_eq!(cursor.to_sequence(), vec!['\0', '2', 'i', 'a']);

        undo!(cursor 1);
        cursor.hit('e');
        assert_eq!(cursor.to_sequence(), vec!['\0', '2', 'i', 'e']);

        undo!(cursor 2);
        hit!(cursor 'o', 'o');
        assert_eq!(cursor.to_sequence(), vec!['\0', '2', 'o', 'o']);

        undo!(cursor 3);
        assert_eq!(cursor.to_sequence(), vec!['\0']);

        hit!(cursor '2', '2', 'u', 'a');
        assert_eq!(
            cursor.to_sequence(),
            vec!['\0', '\0', '2', '\0', '2', 'u', 'a']
        );
        undo!(cursor 4);
        assert_eq!(cursor.to_sequence(), vec!['\0', '\0']);
        undo!(cursor 1);
        assert_eq!(cursor.to_sequence(), vec![]);

        hit!(
            cursor
            'a', 'a', '2', 'a', 'e', 'a', '2', 'f', 'a',
            '2', '2', 'x', 'x', '2', 'i', 'a', '2', '2', '_', 'f',
            '2', 'a', '2', 'a', '_'
        );
        assert_eq!(
            cursor.to_sequence(),
            vec!['f', '\0', '2', 'a', '\0', '2', 'a', '_']
        );

        cursor.clear();
        assert_eq!(cursor.to_sequence(), vec![]);
    }
}

//! Data structure to make handling of sequential
//!     code more convenient.
//!
//! Example
//! ```
//! use afrim_memory::*;
//! use std::{fs, rc::Rc};
//!
//! // Build a TextBuffer
//! let root = Node::default();
//! root.insert(vec!['a', 'f'], "ɑ".to_owned());
//! root.insert(vec!['a', 'f', '1'], "ɑ̀".to_owned());
//!
//! // Bulk insert of data in the TextBuffer
//! let data = vec![vec!["af11", "ɑ̀ɑ̀"], vec!["?.", "ʔ"]];
//! utils::build_map(data);
//!
//! // or directly from a file
//! let data = fs::read_to_string("./data/sample.txt")
//!                   .expect("Failed to load the code file");
//! let data = utils::load_data(&data);
//! utils::build_map(data);
//!
//! // Traverse the tree
//! let node = root.goto('a').and_then(|e| e.goto('f'));
//! assert_eq!(node.unwrap().take(), Some("ɑ".to_owned()));
//!
//! // We initiate our cursor
//! let mut cursor = Cursor::new(Rc::new(root), 10);
//! // We move the cursor to the sequence
//! let code = "af1";
//! code.chars().for_each(|c| {cursor.hit(c);});
//! // We verify the current state
//! assert_eq!(cursor.state(), (Some("ɑ̀".to_owned()), 3, '1'));
//! // We undo the last insertion
//! assert_eq!(cursor.undo(), Some("ɑ̀".to_owned()));
//! ```

#![deny(missing_docs)]

use std::collections::{HashMap, VecDeque};
use std::{cell::RefCell, fmt, rc::Rc};
pub mod utils;

/// Extra information for a `Node`.
#[derive(Clone, Debug)]
pub struct Node {
    children: RefCell<HashMap<char, Rc<Node>>>,
    /// Depth of the node.
    pub depth: usize,
    /// Character holded by the node.
    pub key: char,
    value: RefCell<Option<String>>,
}

impl Default for Node {
    /// Initialize a root node.
    fn default() -> Self {
        Self::new('\0', 0)
    }
}

impl Node {
    /// Initialize a new node.
    pub fn new(key: char, depth: usize) -> Self {
        Self {
            children: HashMap::new().into(),
            depth,
            key,
            value: None.into(),
        }
    }

    /// Insert the sequence in the memory.
    pub fn insert(&self, sequence: Vec<char>, value: String) {
        if let Some(character) = sequence.clone().first() {
            let new_node = Rc::new(Self::new(*character, self.depth + 1));

            self.children
                .borrow()
                .get(character)
                .unwrap_or(&new_node)
                .insert(sequence.into_iter().skip(1).collect(), value);

            self.children
                .borrow_mut()
                .entry(*character)
                .or_insert(new_node);
        } else {
            *self.value.borrow_mut() = Some(value);
        };
    }

    /// Move from the current node to his child.
    pub fn goto(&self, character: char) -> Option<Rc<Self>> {
        self.children.borrow().get(&character).map(Rc::clone)
    }

    /// Extract the value of the node.
    pub fn take(&self) -> Option<String> {
        self.value.borrow().as_ref().map(ToOwned::to_owned)
    }

    /// Return true is the node is at the initial depth.
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }
}

/// The Cursor permit to keep a track of the move in the memory.
#[derive(Clone)]
pub struct Cursor {
    buffer: VecDeque<Rc<Node>>,
    root: Rc<Node>,
}

impl fmt::Debug for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_sequence().fmt(f)
    }
}

impl Cursor {
    /// Initialize the cursor.
    pub fn new(root: Rc<Node>, capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            root,
        }
    }

    /// Enter a character in the sequence and return his corresponding out.
    pub fn hit(&mut self, character: char) -> Option<String> {
        let node = self
            .buffer
            .iter()
            .last()
            .unwrap_or(&Rc::new(Node::default()))
            .goto(character)
            .or_else(|| {
                // We end the current sequence
                self.insert(Rc::new(Node::default()));
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

    /// Remove the last node and return his corresponding out.
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

    /// Return the current state of the cursor.
    pub fn state(&self) -> (Option<String>, usize, char) {
        self.buffer
            .iter()
            .last()
            .map(|n| (n.take(), n.depth, n.key))
            .unwrap_or_default()
    }

    /// Return the current sequence in the cursor.
    pub fn to_sequence(&self) -> Vec<char> {
        self.buffer.iter().map(|node| node.key).collect()
    }

    /// Clear the memory of the cursor.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Verify if the cursor is empty.
    pub fn is_empty(&self) -> bool {
        return self.buffer.iter().filter(|c| c.key != '\0').count() == 0;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_node() {
        use crate::Node;

        let root = Node::default();

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
        use crate::{utils, Cursor};
        use std::{fs, rc::Rc};

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

        let data = fs::read_to_string("./data/sample.txt").unwrap();
        let root = utils::build_map(utils::load_data(&data));

        let mut cursor = Cursor::new(Rc::new(root), 8);

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
        assert!(cursor.is_empty());
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

        assert_eq!(
            format!("{:?}", cursor),
            format!("{:?}", cursor.to_sequence())
        );

        cursor.clear();
        assert_eq!(cursor.to_sequence(), vec![]);
    }
}

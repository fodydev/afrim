#![deny(missing_docs)]
//! Data structure to make handling of sequential code more convenient.
//!
//! It takes sequential codes and generates a text buffer that will be used to easily get a
//! corresponding character through an input.
//!
//! # Notes
//! - sequence: A sequential code corresponding to a character.
//!     Eg. af1 = "ɑ̀"
//! - input: The user input (or a set of sequences).
//!     Eg. ngaf7 nkwe2e2 ka7meru7n
//! - text buffer: The memory where our text data will be stored.
//! - node: A node in the text buffer.
//!
//! # Example
//!
//! ```
//! use afrim_memory::{Node, utils};
//!
//! // Builds a TextBuffer.
//! let text_buffer = Node::default();
//! text_buffer.insert(vec!['a', 'f'], "ɑ".to_owned());
//! text_buffer.insert(vec!['a', 'f', '1'], "ɑ̀".to_owned());
//!
//! // Bulk insertion of data in the TextBuffer.
//! let data = vec![vec!["af11", "ɑ̀ɑ̀"], vec!["?.", "ʔ"]];
//! let text_buffer = utils::build_map(data);
//!
//! // Traverses the tree.
//! let node = text_buffer.goto('a').and_then(|node| node.goto('f')).and_then(|node| node.goto('1')).and_then(|node| node.goto('1'));
//! assert_eq!(node.unwrap().take(), Some("ɑ̀ɑ̀".to_owned()));
//! ```
//!
//! # Example: in reading data through a file
//!
//! ```no_run
//! use afrim_memory::utils;
//!
//! // Import data from a string.
//! let data = "a1 à\ne2 é";
//! let data = utils::load_data(data);
//! let text_buffer = utils::build_map(data);
//! ```
//!
//! # Example: with the usage of a cursor
//!
//! ```
//! use afrim_memory::{Cursor, Node};
//! use std::rc::Rc;
//!
//! // Build a TextBuffer.
//! let text_buffer = Node::default();
//! text_buffer.insert(vec!['i', '-'], "ɨ".to_owned());
//! text_buffer.insert(vec!['i', '-', '3'], "ɨ̄".to_owned());
//!
//! // Builds the cursor.
//! let memory = Rc::new(text_buffer);
//! let mut cursor = Cursor::new(memory, 16);
//!
//! // Moves the cursor through the input.
//! let input = "i-3";
//! input.chars().for_each(|c| { cursor.hit(c); });
//! // Verify the current state.
//! assert_eq!(cursor.state(), (Some("ɨ̄".to_owned()), 3, '3'));
//!
//! // Undo the last insertion.
//! assert_eq!(cursor.undo(), Some("ɨ̄".to_owned()));
//! // Verify the current state.
//! assert_eq!(cursor.state(), (Some("ɨ".to_owned()), 2, '-'));
//! ```
//!
//! [`TextBuffer`]: https://en.wikipedia.org/wiki/Text_buffer

use std::collections::{HashMap, VecDeque};
use std::{cell::RefCell, fmt, rc::Rc};
pub mod utils;

/// A node in the text buffer.
///
/// ```text
///              0 ----------------> The root node
///             / \
///           'g' 's' -------------> Node: Rc<Node>
///           /     \
///   "ɣ" = '+'     'h' -----------> Node: Rc<Node>
///                   \
///                   '+' = "ʃ" ---> Node that holds a value
/// ```
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
    /// Create a root node.
    ///
    /// A root node always holds a null character as key and is recommanded to use
    /// to initialize the text buffer. You should always use it to create a text buffer because the
    /// internal code can change.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::Node;
    ///
    /// // It's recommanded to use it, to initialize your text buffer.
    /// let text_buffer = Node::default();
    /// // Not recommanded.
    /// let another_text_buffer = Node::new('\0', 0);
    ///
    /// assert!(text_buffer.is_root());
    /// assert!(another_text_buffer.is_root());
    /// ```
    fn default() -> Self {
        Self::new('\0', 0)
    }
}

impl Node {
    /// Initializes a new node in the text buffer.
    ///
    /// Can also be used to initialize the text buffer (not recommanded).
    /// Uses [`Node::default`](crate::Node::default) instead.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::Node;
    ///
    /// let text_buffer = Node::new('\0', 0);
    ///
    /// // You cannot assign directly a value to a node.
    /// // But, an alternative is as below.
    /// let node = Node::new('u', 0);
    /// node.insert(vec![], "ʉ̠̀".to_owned());
    /// assert_eq!(node.take(), Some("ʉ̠̀".to_owned()));
    /// ```
    ///
    /// **Note**: Early, [`Node::new`](crate::Node::new) was the only way to initialize a text
    /// buffer but it has been replaced by [`Node::default`](crate::Node::default)
    /// which is now more adapted for this use case.
    pub fn new(key: char, depth: usize) -> Self {
        Self {
            children: HashMap::new().into(),
            depth,
            key,
            value: None.into(),
        }
    }

    /// Inserts a sequence in the text buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::Node;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['.', 't'], "ṫ".to_owned());
    ///
    /// let node = text_buffer.goto('.').and_then(|node| node.goto('t'));
    /// assert_eq!(node.unwrap().take(), Some("ṫ".to_owned()));
    /// ```
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

    /// Moves from the current node to his child.
    ///
    /// Useful to go through a sequence.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::Node;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['o', '/'], "ø".to_owned());
    /// text_buffer.insert(vec!['o', '*'], "ɔ".to_owned());
    /// text_buffer.insert(vec!['o', '1'], "ò".to_owned());
    /// text_buffer.insert(vec!['o', '*', '~'], "ɔ̃".to_owned());
    ///
    /// // let sequence = ['o', '*', '~'];
    /// let node = text_buffer.goto('o').unwrap();
    /// assert_eq!(node.take(), None);
    /// let node = node.goto('*').unwrap();
    /// assert_eq!(node.take(), Some("ɔ".to_owned()));
    /// let node = node.goto('~').unwrap();
    /// assert_eq!(node.take(), Some("ɔ̃".to_owned()));
    /// ```
    pub fn goto(&self, character: char) -> Option<Rc<Self>> {
        self.children.borrow().get(&character).map(Rc::clone)
    }

    /// Extracts the value of the node.
    ///
    /// A node in the text buffer don't always holds a value.
    /// Hence, his value is optional.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::Node;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['1', 'c'], "c̀".to_string());
    ///
    /// let node = text_buffer.goto('1').unwrap();
    /// assert_eq!(node.take(), None);
    /// let node = node.goto('c').unwrap();
    /// assert_eq!(node.take(), Some("c̀".to_owned()));
    /// ```
    pub fn take(&self) -> Option<String> {
        self.value.borrow().as_ref().map(ToOwned::to_owned)
    }

    /// Returns true is the node is at the initial depth.
    ///
    /// Useful when dealing with the [`Cursor`](crate::Cursor).
    /// Will permit to know the beginning and the end of a sequence.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['e', '2' ], "é".to_owned());
    /// text_buffer.insert(vec!['i', '7' ], "ǐ".to_owned());
    ///
    /// assert!(text_buffer.is_root());
    /// let node = text_buffer.goto('e').unwrap();
    /// assert!(!node.is_root());
    ///
    /// ```
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }
}

/// The Cursor permits to keep a track of the different positions while moving in
/// the text buffer.
///
/// ```text
/// '\0' - 'k' - '\0' - 'w' - '\0'  '\0'  '\0' - '\'' - 'n' - 'i' - '7' |--> 0
///                            |    /|    /                             |
///                            |   / |   /                              |
///                           'e' / 'e' /                               |--> 1
///                            | /   | /                                |
///                           '2'   '2'                                 |--> 2
///                                                                     |
///                                                                     | depth
///                                                                     v
/// ```
///
/// # Example
///
/// ```
/// use afrim_memory::{Cursor, Node};
/// use std::rc::Rc;
///
/// let text_buffer = Node::default();
/// text_buffer.insert(vec!['e', '2'], "é".to_owned());
/// text_buffer.insert(vec!['i', '7'], "ǐ".to_owned());
///
/// // We build our cursor.
/// let memory = Rc::new(text_buffer);
/// let mut cursor = Cursor::new(memory, 16);
/// let input = "nkwe2e2'ni7";
/// input.chars().for_each(|c| { cursor.hit(c); });
///
/// assert_eq!(
///     cursor.to_sequence(),
///     vec![
///         'k', '\0', 'w', '\0', 'e', '2', '\0', 'e', '2', '\0',
///         '\'', '\0', 'n', '\0', 'i', '7'
///     ]
/// );
/// ```
///
/// Note the partitioning of this input. The cursor can browse through the memory based
/// on an input and save a track of his positions. It's useful when we want handle
/// backspace operations in an input method engine.
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
    /// Initializes the cursor of a text buffer.
    ///
    /// `capacity` is the number of hit that the cursor can track. The cursor follows the FIFO
    /// rule. If the capacity is exceeded, the oldest hit will be discarded.
    ///
    /// **Note**: Be careful when you set this capacity. We recommend to select a capacity equal or
    /// greater than the maximun sequence length that you want handle.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// let memory = Rc::new(text_buffer);
    ///
    /// // A cursor of our text buffer.
    /// let cursor = Cursor::new(memory, 16);
    /// ```
    ///
    /// **Note**: It's recommended to initialize the text buffer with
    /// [`Node::default`](crate::Node::default) to evict unexpected behaviors.
    pub fn new(root: Rc<Node>, capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            root,
        }
    }

    /// Enters a character in the sequence and returns his corresponding out.
    ///
    /// Permits to simulate the user typing in the input method engine.
    /// For each character entered, the cursor will move through the text buffer in looking of the
    /// corresponding sequence. If the sequence is got (end on a value), his value will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['o', 'e'], "œ".to_owned());
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut cursor = Cursor::new(memory, 16);
    /// // let input= "coeur";
    /// assert_eq!(cursor.hit('c'), None);
    /// assert_eq!(cursor.hit('o'), None);
    /// assert_eq!(cursor.hit('e'), Some("œ".to_owned()));
    /// assert_eq!(cursor.hit('u'), None);
    /// assert_eq!(cursor.hit('r'), None);
    ///
    /// assert_eq!(cursor.to_sequence(), vec!['\0', 'c', '\0', 'o', 'e', '\0', 'u', '\0', 'r']);
    /// ```
    ///
    /// **Note**:
    /// - The `\0` at the index 0, marks the beginning of a new sequence and the end of a
    /// old. It also represents the root node.
    /// - A character don't necessary need to be in the text buffer. The cursor will create a
    /// tempory node to represent it in his internal memory. All characters not present in the text
    /// buffer will be at the same depth that the root node.
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

    /// Removes the last node and returns his corresponding out.
    /// Or simplily, undo the previous hit.
    ///
    /// Useful to simulate a backspace operation.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['o', 'e'], "œ".to_owned());
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut cursor = Cursor::new(memory, 16);
    /// // let input = "coeur";
    /// assert_eq!(cursor.hit('c'), None);
    /// assert_eq!(cursor.hit('o'), None);
    /// assert_eq!(cursor.hit('e'), Some("œ".to_owned()));
    /// assert_eq!(cursor.hit('u'), None);
    /// assert_eq!(cursor.hit('r'), None);
    ///
    /// // Undo
    /// assert_eq!(cursor.undo(), None);
    /// assert_eq!(cursor.undo(), None);
    /// assert_eq!(cursor.undo(), Some("œ".to_owned()));
    /// assert_eq!(cursor.undo(), None);
    /// assert_eq!(cursor.undo(), None);
    ///
    /// assert_eq!(cursor.to_sequence(), vec!['\0']);
    /// ```
    ///
    /// **Note**: Look at the `\0` at the end. It represents the root node, and the start of a
    /// new sequence. Even if you remove it until obtain an empty buffer, the cursor will add it
    /// before each new sequence. You can considere it as a delimiter between two sequences. But if
    /// you want clear or verify if the buffer is empty, you can use [Cursor::clear](crate::Cursor::clear) or [Cursor::is_empty](crate::Cursor::is_empty).
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

    /// Returns the current state of the cursor.
    ///
    /// Permits to know the current position in the memory and also the last hit.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['o', '/'], "ø".to_owned());
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut cursor = Cursor::new(memory, 8);
    /// // The cursor starts always at the root node.
    /// assert_eq!(cursor.state(), (None, 0, '\0'));
    /// cursor.hit('o');
    /// assert_eq!(cursor.state(), (None, 1, 'o'));
    /// ```
    pub fn state(&self) -> (Option<String>, usize, char) {
        self.buffer
            .iter()
            .last()
            .map(|n| (n.take(), n.depth, n.key))
            .unwrap_or_default()
    }

    /// Returns the current sequence in the cursor.
    ///
    /// It's always useful to know what is inside the memory of the cursor for debugging / logging.
    /// The
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// text_buffer.insert(vec!['.', '.', 'z'], "z̈".to_owned());
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut cursor = Cursor::new(memory, 8);
    /// "z..z".chars().for_each(|c| { cursor.hit(c); });
    ///
    /// assert_eq!(cursor.to_sequence(), vec!['\0', 'z', '\0', '.', '.', 'z']);
    /// ```
    pub fn to_sequence(&self) -> Vec<char> {
        self.buffer.iter().map(|node| node.key).collect()
    }

    /// Clear the memory of the cursor.
    ///
    /// In clearing the internal buffer, all the tracking information will be lost.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// let memory = Rc::new(text_buffer);
    /// let mut cursor = Cursor::new(memory, 8);
    ///
    /// "hello".chars().for_each(|c| { cursor.hit(c); });
    /// assert!(!cursor.is_empty());
    ///
    /// cursor.clear();
    /// assert!(cursor.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Verify if the cursor is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_memory::{Cursor, Node};
    /// use std::rc::Rc;
    ///
    /// let text_buffer = Node::default();
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut cursor = Cursor::new(memory, 8);
    /// assert!(cursor.is_empty());
    ///
    /// cursor.hit('a');
    /// assert!(!cursor.is_empty());
    /// ```
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
        use std::rc::Rc;

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

        let data = include_str!("../data/sample.txt");
        let root = utils::build_map(utils::load_data(data));

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

#![deny(missing_docs)]
//! Set of tools to facilitate the loading of data.

use crate::Node;

/// Load the sequential codes from a plain text and returns it.
///
/// # Example
///
/// ```
/// use afrim_memory::{Cursor, Node, utils};
/// use std::rc::Rc;
///
/// let text_buffer = Node::default();
/// let data = utils::load_data(r#"
/// ..a	   	ä
/// ..af	ɑ̈
/// ..ai	ɛ̈
/// "#);
/// data.iter().for_each(|d| { text_buffer.insert(d[0].chars().collect(), d[1].to_owned()); });
/// let memory = Rc::new(text_buffer);
///
/// let mut cursor = Cursor::new(memory, 8);
/// "..af".chars().for_each(|c| { cursor.hit(c); });
///
/// assert_eq!(cursor.state(), (Some("ɑ̈".to_owned()), 4, 'f'));
///```
pub fn load_data(data: &str) -> Vec<Vec<&str>> {
    let data = data
        .trim()
        .split('\n')
        .map(|line| {
            line.split_whitespace()
                .filter(|token| !token.is_empty())
                .take(2)
                .collect()
        })
        .collect();

    data
}

/// Build a map from a list of sequential codes.
///
/// # Example
///
/// ```
/// use afrim_memory::{Cursor, Node, utils};
/// use std::rc::Rc;
///
/// let data = utils::load_data(r#"
/// ..a     ä
/// ..af    ɑ̈
/// ..ai    ɛ̈
/// "#);
/// let text_buffer = utils::build_map(data);
/// let memory = Rc::new(text_buffer);
///
/// let mut cursor = Cursor::new(memory, 8);
/// "..af".chars().for_each(|c| { cursor.hit(c); });
///
/// assert_eq!(cursor.state(), (Some("ɑ̈".to_owned()), 4, 'f'));
///```
pub fn build_map(data: Vec<Vec<&str>>) -> Node {
    let root = Node::default();

    data.iter().for_each(|e| {
        root.insert(e[0].chars().collect(), e[1].to_owned());
    });

    root
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_load_data() {
        use crate::utils;

        let data = fs::read_to_string("./data/sample.txt").unwrap();

        utils::load_data(&data)
            .iter()
            .for_each(|pair| assert_eq!(pair.len(), 2));
    }

    #[test]
    fn test_build_map() {
        use crate::utils;

        let data = vec![vec!["af11", "ɑ̀ɑ̀"], vec!["?.", "ʔ"]];
        utils::build_map(data);

        let data = fs::read_to_string("./data/sample.txt").unwrap();
        let data = utils::load_data(&data);

        utils::build_map(data);
    }
}

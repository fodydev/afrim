use std::collections::HashMap;
use std::{fs, cell::RefCell, rc::Weak, rc::Rc};

#[derive(Debug)]
pub struct Node<'a> {
    neighbors: RefCell<HashMap<char, Rc<Node<'a>>>>,
    out: RefCell<Option<String>>
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Node {
            neighbors: HashMap::new().into(),
            out: None.into()
        }
    }

    pub fn insert(&self, path: Vec<char>, out: String) {
        if let Some(character) = path.clone().first() {
            let new_node = Rc::new(Self::new());

            self.neighbors.borrow()
                .get(character).unwrap_or(&new_node)
                .insert(path.into_iter().skip(1).collect(), out);

            self.neighbors.borrow_mut().entry(*character).or_insert(new_node);
        } else {
            *self.out.borrow_mut() = Some(out);
        };
    }

    pub fn get(&self, character: char) -> Option<Rc<Self>> {
        self.neighbors.borrow().get(&character).map(Rc::clone)
    }

    pub fn take(&self) -> Option<String> {
        self.out.borrow().as_ref().map(ToOwned::to_owned)
    }
}

pub fn load_data(file_path: &str) -> Vec<Vec<String>> {
    let data = fs::read_to_string(file_path).unwrap();
    data.split('\n')
        .map(|line| line.split_whitespace()
            .filter(|token| !token.is_empty())
            .map(ToOwned::to_owned)
            .collect())
        .collect()
}

pub fn build_map(data: Vec<[&str; 2]>) -> Node {
    let root = Node::new();

    data.iter().for_each(|e| {
        root.insert(e[0].chars().collect(), e[1].to_owned());
    });

    root
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
        use crate::load_data;

        generate_data_file();

        load_data("../tmp/data.txt").iter().for_each(|pair| assert_eq!(pair.len(), 2));
    }

    #[test]
    fn test_build_map() {
        use crate::build_map;
        use crate::load_data;

        let data = vec![["af11", "ɑ̀ɑ̀"], ["?.", "ʔ"]];
        build_map(data);

        let data = load_data("../tmp/data.txt");
        build_map(data.iter().map(|e| [e[0].as_str(), e[1].as_str()]).collect());
    }

    #[test]
    fn test_node() {
        use crate::Node;

        let root = Node::new();
        root.insert(vec!['a','f'], "ɑ".to_owned());
        root.insert(vec!['a','f','1'], "ɑ̀".to_owned());

        assert!(root.get('a').is_some());
        assert!(root.get('b').is_none());

        let node = root.get('a').and_then(|e| e.get('f'));
        assert_eq!(node.as_ref().unwrap().take(), Some("ɑ".to_owned()));

        let node = node.and_then(|e| e.get('1'));
        assert_eq!(node.unwrap().take(), Some("ɑ̀".to_owned()));
    }
}
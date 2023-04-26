use std::collections::HashMap;
use std::{fs, cell::RefCell, rc::Weak, rc::Rc};

struct Cursor<'a> {
    stack: Vec<&'a str>,
    path: &'a Path<'a>,
}

#[derive(Debug)]
struct Path<'a> {
    neighbors: RefCell<HashMap<char, Rc<Path<'a>>>>,
    out: RefCell<Option<String>>
}

impl<'a> Path<'a> {
    fn new() -> Self {
        Path {
            neighbors: HashMap::new().into(),
            out: None.into()
        }
    }

    fn insert(&self, code: Vec<char>, out: String) {
        if let Some(character) = code.clone().first() {
            let new_path = Rc::new(Self::new());

            self.neighbors.borrow()
                .get(character).unwrap_or(&new_path)
                .insert(code.into_iter().skip(1).collect(), out);

            self.neighbors.borrow_mut().entry(*character).or_insert(new_path);
        } else {
            *self.out.borrow_mut() = Some(out);
        };
    }

    fn get(&self, character: char) -> Option<Rc<Self>> {
        self.neighbors.borrow().get(&character).map(Rc::clone)
    }

    fn take(&self) -> Option<String> {
        self.out.borrow().as_ref().map(ToOwned::to_owned)
    }
}

fn run() {
    unimplemented!();
}

fn load_data() -> Vec<Vec<String>> {
    let data = fs::read_to_string("data/clafrica_codes.txt").unwrap();
    data.split('\n')
        .map(|line| line.split_whitespace()
            .filter(|token| !token.is_empty())
            .map(ToOwned::to_owned)
            .collect())
        .collect()
}

fn build_map(data: Vec<[&str; 2]>) -> Path {
    let root = Path::new();

    data.iter().for_each(|e| {
        root.insert(e[0].chars().collect(), e[1].to_owned());
    });

    root
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_data() {
        use crate::load_data;

        load_data().iter().for_each(|pair| assert_eq!(pair.len(), 2));
    }

    #[test]
    fn test_build_map() {
        use crate::build_map;
        use crate::load_data;

        let data = vec![["af11", "ɑ̀ɑ̀"], ["?.", "ʔ"]];
        build_map(data);

        let data = load_data();
        build_map(data.iter().map(|e| [e[0].as_str(), e[1].as_str()]).collect());
    }

    #[test]
    fn test_path() {
        use crate::Path;

        let root = Path::new();
        root.insert(vec!['a','f'], "ɑ".to_owned());
        root.insert(vec!['a','f','1'], "ɑ̀".to_owned());

        assert!(root.get('a').is_some());
        assert!(root.get('b').is_none());

        let path = root.get('a').and_then(|e| e.get('f'));
        assert_eq!(path.as_ref().unwrap().take(), Some("ɑ".to_owned()));

        let path = path.and_then(|e| e.get('1'));
        assert_eq!(path.unwrap().take(), Some("ɑ̀".to_owned()));
    }
}
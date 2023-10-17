use crate::Node;

/// Load the sequential code from a plain text.
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

/// Build a map from the sequential code.
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

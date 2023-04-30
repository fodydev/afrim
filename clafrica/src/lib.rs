use clafrica_lib::bst;

struct Cursor<'a> {
    stack: Vec<&'a str>,
    node: &'a bst::Node,
}

fn run() {
    unimplemented!();
}

#[cfg(test)]
mod tests {}

use clafrica_lib::Node;

struct Cursor<'a> {
    stack: Vec<&'a str>,
    node: &'a Node<'a>,
}

fn run() {
    unimplemented!();
}

#[cfg(test)]
mod tests {}

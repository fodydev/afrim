use clafrica_lib::text_buffer;

struct Cursor<'a> {
    stack: Vec<&'a str>,
    node: &'a text_buffer::Node,
}

fn run() {
    unimplemented!();
}

#[cfg(test)]
mod tests {}

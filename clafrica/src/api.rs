pub trait Frontend {
    fn update_screen(&mut self, _screen: (u64, u64)) {}
    fn update_position(&mut self, _position: (f64, f64)) {}
    fn update_text(&mut self, _text: &str) {}
    fn add_predicate(&mut self, _remaining_code: &str, _text: &str) {}
    fn clear_predicates(&mut self) {}
}

pub struct None;

impl Frontend for None {}

pub struct Console;

impl Frontend for Console {
    fn update_text(&mut self, text: &str) {
        println!("text: {:?}", text);
    }
    fn add_predicate(&mut self, remaining_code: &str, text: &str) {
        println!("predicate: {} ~{}", text, remaining_code);
    }
}

#[test]
fn test_console() {
    let mut console = Console;
    let mut none = None;
    console.update_screen((0, 0));
    console.update_position((0.0, 0.0));
    console.update_text("hello");
    console.add_predicate("_", "10/12/2003");
    console.clear_predicates();
    none.update_text("hello");
}

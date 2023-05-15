pub trait Frontend {
    fn update_screen(&mut self, _screen: (u64, u64)) {}
    fn update_position(&mut self, _position: (f64, f64)) {}
    fn update_text(&mut self, _text: Vec<char>) {}
}

pub struct None;

impl Frontend for None {}

pub struct Console;

impl Frontend for Console {
    fn update_text(&mut self, text: Vec<char>) {
        println!("{:?}", text);
    }
}

#[test]
fn test_console() {
    let mut console = Console;
    let mut none = None;
    console.update_screen((0, 0));
    console.update_position((0.0, 0.0));
    console.update_text(vec!['h', 'e', 'l', 'l', 'o']);
    none.update_text(vec!['h', 'e', 'l', 'l', 'o']);
}

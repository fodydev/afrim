pub trait Frontend {
    fn update_screen(&mut self, _screen: (u64, u64)) {}
    fn update_position(&mut self, _position: (f64, f64)) {}
    fn update_text(&mut self, _text: Vec<char>) {}
}

pub struct Console;

impl Frontend for Console {
    fn update_text(&mut self, text: Vec<char>) {
        println!("{:?}", text);
    }
}

#[test]
fn test_console() {
    let mut console = Console;
    console.update_screen((0, 0));
    console.update_position((0.0, 0.0));
    Frontend::update_text(&mut console, vec!['h', 'e', 'l', 'l', 'o']);
    console.update_text(vec!['h', 'e', 'l', 'l', 'o']);
}

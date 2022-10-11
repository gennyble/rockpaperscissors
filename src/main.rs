use smitten::{self, Color, Smitten};

fn main() {
    let mut smitty = Smitten::new((640, 480), "Square", 24);

    let rock = Color::grey(0.5);
    let paper = Color::rgb(0.7, 0.7, 0.4);
    let scissors = Color::rgb(0.8, 0.3, 0.3);

    loop {
        let _events = smitty.events();

        smitty.clear();

        smitty.rect((2.0, 2.0), (1.0, 1.0), rock);
        smitty.rect((-2.0, 2.0), (1.0, 1.0), paper);
        smitty.rect((2.0, -2.0), (1.0, 1.0), scissors);

        smitty.swap();
    }
}

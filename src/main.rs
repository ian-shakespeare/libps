use std::io::{self, BufRead, Write};

use libps::{FileObject, Interpreter};

fn main() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();

    output.write_all(b"libPS 0.0.0\n")?;
    let mut interpreter = Interpreter::new(output);

    loop {
        interpreter.push_string(">>> ".into());
        let _ = interpreter.print(); // TODO: handle this
        let _ = interpreter.flush();

        let mut buf = String::new();
        input.read_line(&mut buf)?;

        interpreter.push_file(FileObject::from(buf));

        if let Err(e) = interpreter.exec() {
            panic!("{}", e.to_string());
        }
    }
}

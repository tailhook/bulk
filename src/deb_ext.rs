use std::io::{self, Write};


pub trait WriteDebExt: Write {
    fn write_kv(&mut self, key: &str, value: &str) -> io::Result<()> {
        write!(self, "{}: {}\n", key, _control_multiline(value))
    }
    fn write_kv_lines<I, E>(&mut self, key: &str, lines: I) -> io::Result<()>
        where I: Iterator<Item=E>, E: AsRef<str>
    {
        try!(write!(self, "{}:\n", key));
        for line in lines {
            try!(write!(self, " {}\n", _control_multiline(line.as_ref())));
        }
        Ok(())
    }
}

fn _control_multiline(val: &str) -> String {
    val
        .replace("\n\n", "\n.\n")
        .replace("\n", "\n ")
}

impl<T: Write> WriteDebExt for T {}

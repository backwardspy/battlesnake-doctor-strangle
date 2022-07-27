use std::fmt;

pub struct Indent(pub u64, pub u64);

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.0 * 4 + self.1 {
            write!(f, "█")?;
        }
        write!(f, "▶ ")
    }
}

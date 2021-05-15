use std::fmt;

pub struct Session {
    ph_number: String,
}

impl Session {
    pub fn new(ph_number: &str) -> Self {
        Self {
            ph_number: ph_number.to_owned(),
        }
    }

    // example methods
    #[allow(dead_code)]
    pub fn init(&mut self) {
        unimplemented!();
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        unimplemented!();
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ph_number)
    }
}

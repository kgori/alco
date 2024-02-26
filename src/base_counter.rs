pub struct BaseCounter {
    a_count: u32,
    c_count: u32,
    g_count: u32,
    t_count: u32,
    ref_count: u32,
    alt_count: u32,
    ref_base: char,
    alt_base: char,
}

impl BaseCounter {
    pub fn new(ref_base: char, alt_base: char) -> Self {
        Self {
            a_count: 0,
            c_count: 0,
            g_count: 0,
            t_count: 0,
            ref_count: 0,
            alt_count: 0,
            ref_base,
            alt_base
        }
    }

    pub fn add(&mut self, base: char) {
        match base {
            'A' => self.a_count += 1,
            'C' => self.c_count += 1,
            'G' => self.g_count += 1,
            'T' => self.t_count += 1,
            _ => (),
        }
        if base == self.ref_base { self.ref_count += 1; }
        if base == self.alt_base { self.alt_count += 1; }
    }

    pub fn total(&self) -> u32 {
        self.a_count + self.c_count + self.g_count + self.t_count
    }
}

impl std::fmt::Display for BaseCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.total(), self.a_count, self.c_count, self.g_count, self.t_count, self.ref_count, self.alt_count
        )
    }
}
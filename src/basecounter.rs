// BaseCounter is a structure that tracks the number of ACGT nucleotides it
// has seen.

pub struct BaseCounter {
    n_a: u32,
    n_c: u32,
    n_g: u32,
    n_t: u32
}

impl BaseCounter {
    pub fn update(&mut self, base: char) -> () {
        match base {
            'A' => self.n_a += 1,
            'C' => self.n_c += 1,
            'G' => self.n_g += 1,
            'T' => self.n_t += 1,
            _ => (),
        }
    }

    pub fn sum(&self) -> u32 {
        return self.n_a + self.n_c + self.n_g + self.n_t;
    }

    pub fn new() -> BaseCounter {
        BaseCounter{n_a: 0, n_c: 0, n_g: 0, n_t: 0}
    }

    pub fn write(&self) -> String {
        return format!("{}\t{}\t{}\t{}\t{}",
                       self.n_a, self.n_c, self.n_g, self.n_t, self.sum());
    }

    #[allow(dead_code)]
    pub fn has_data(&self) -> bool {
        return self.n_a > 0 || self.n_c > 0 || self.n_g > 0 || self.n_t > 0;
    }
}
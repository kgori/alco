use std::error::Error;

#[derive(Clone, Debug)]
pub struct Variant {
    pub chr: String,
    pub pos: u32,
    pub refbase: char,
    pub altbase: char,
}

impl Variant {
    pub fn from_csv_record(record: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        let chr = record.get(0).ok_or("Missing CHROM in CSV record")?.to_owned();
        let pos: u32 = record.get(1).ok_or("Missing POS in CSV record")?.parse()?;
        let refbase = record.get(2).ok_or("Missing REF in CSV record")?.chars().next().unwrap();
        let altbase = record.get(3).ok_or("Missing ALT in CSV record")?.chars().next().unwrap();
        Ok(
            Self {
                chr,
                pos,
                refbase,
                altbase,
            }
        )
    }

    pub fn pos_zero_based(&self) -> u32 {
        self.pos - 1
    }

    pub fn pos_one_based(&self) -> u32 {
        self.pos
    }
}
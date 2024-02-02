#[derive(Debug, Clone, clap::ValueEnum, Copy)]
pub enum Method {
    /// => 9AFoh, wGRLC, knj9y, ... (--nbits => 5)
    Random,
    /// => de2662a9-fb02-4686-b556-0aca36c0e087
    Uuid,
    /// => 2023-03-04-22-26-42-222655555  (--delimiter => -)
    Time,
    /// => 1, 2, 3, ... (--start => 1)
    Num,
    /// => 001, 002, 003, ... (--nbits => 3)
    Znum,
    /// => X.jpg --> <Prefix><Delimiter>X.jpg
    Prefix,
    /// => X.jpg --> X<Delimiter><Append>.jpg
    Append,
    /// TODO: OS problem. aBcDe123.txt --> ABCDE123.txt.
    Uppercase,
    /// TODO: OS problem. aBcDe123.txt --> abcde123.txt.
    Lowercase,
    // /// TODO
    // Capitalized,
    // /// TODO
    // Snake,
    // /// TODO
    // Replace,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "Random" => Self::Random,
            "Uuid" => Self::Uuid,
            "Time" => Self::Time,
            "Numbered" => Self::Num,
            "Zero-Numbered" => Self::Znum,
            "Prefix" => Self::Prefix,
            "Append" => Self::Append,
            "Uppercase" => Self::Uppercase,
            "Lowercase" => Self::Lowercase,
            _ => todo!(),
        }
    }
}

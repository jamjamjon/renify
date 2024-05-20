#[derive(Debug, Clone, clap::ValueEnum)]
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
    // TODO: OS problem, unsafe
    // windows: unsupported ,NTFS
    // Linux: supported, ext4
    // maxos: unsupported, APFS
    // Uppercase,
    // Lowercase,
    // Capitalized,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "Random" => Self::Random,
            "Uuid" => Self::Uuid,
            "Time" => Self::Time,
            "Numbered" => Self::Num,
            "ZeroNumbered" => Self::Znum,
            "Prefix" => Self::Prefix,
            "Append" => Self::Append,
            // "Uppercase" => Self::Uppercase,
            // "Lowercase" => Self::Lowercase,
            _ => todo!(),
        }
    }
}

# Renify
A simple cli tool for batch renaming files and folders, written in Rust.
- [x] support files
- [x] support folders
- [ ] **symlinks are not supported!**



![Example GIF](assets/demo.gif)

# Installation
You can install with `pip` or `cargo`
```
cargo install renify
pip install renify
```

# Usage

For those new to Renify, consider using the following code for interactive mode:
```bash
renify -i <File or Folder Path>
```

Or you can use:
```bash
renify -i <File or Folder Path> --target file --method znum --nbits 5 --recursive false --start 1 -y
```

You can revert to the previous state of the modifications by using:
```bash
renify -i . --roll
```

You can get help from:
```bash
renify --help
```


# Methods
- [x] **random:** Sample a u8, uniformly distributed over ASCII letters and numbers: a-z, A-Z and 0-9. `9AFoh, wGRLC, knj9y, ...`
- [x] **uuid:** Uuid4. `de2662a9-fb02-4686-b556-0aca36c0e087`
- [x] **time:** Local time now. `2023-03-04-22-26-42-222655555`
- [x] **num:**  Numbers start from `--start` (1 by default). `1, 2, 3, ...`
- [x] **znum:** Numbers with left zero padding start from `--start` (1 by default). `001, 002, 003, ...`
- [x] **prefix:** Add a prefix string to the file stem, along with a delimiter. `X.jpg => [--with][--delimiter]X.jpg`
- [x] **append:** Append a delimiter and a string after the file stem. `X.jpg => X[--delimiter][--with].jpg`
- [ ] **uppercase:** To uppercase. `aBcDe123.txt => ABCDE123.txt`
- [ ] **lowercase:** To lowercase. `aBcDe123.txt => abcde123.txt`

# Note that
Renify will set `--indiscriminate false` to make sure that the file stems stay consistent. This means that if you have files with the same stems in the same folder, they'll still look the same even after you rename them. Certainly, you can use `--indiscriminate` to treat each file as an independent entity without considering its relationship with other files.

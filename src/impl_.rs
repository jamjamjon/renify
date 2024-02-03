use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

use crate::{Cli, Method, Target};

const STR_NOT_ALLOWED: &str = "<>:/\"|?*'`"; // TODO
const HELLO: &str = "👋 Welcome to Renify\n";

impl Cli {
    pub fn run(&mut self) -> Result<()> {
        println!("{}", console::Style::new().white().bright().apply_to(HELLO));

        let theme = Self::build_theme();
        let s = if self.roll {
            self.roll_back()?;
            "You have successfully reverted to the previous state!"
        } else {
            self.rename(&theme)?;
            "Done!"
        };
        println!(
            "\n\n{}{}",
            console::Emoji("👏 ", ""),
            console::Style::new().white().bright().apply_to(s)
        );
        println!(
            "{}{} {} {}",
            console::Emoji("✨ ", ""),
            console::Style::new()
                .white()
                .bright()
                .apply_to("Note that you can always use"),
            console::Style::new()
                .color256(49)
                .bright()
                .apply_to("`renify -i . --roll`"),
            console::Style::new()
                .white()
                .bright()
                .apply_to("to revert to the previous state of the modifications."),
        );

        Ok(())
    }

    pub fn rename(&mut self, theme: &ColorfulTheme) -> Result<()> {
        // source & target
        let source_type = self.check_source()?;
        self.ask_target(source_type, theme)?;
        let ys = self.fetch_targets(&self.input)?;

        // continue?
        if ys.is_empty() {
            self.status_log(
                false,
                "Found",
                &format!("{:?} x0", self.target.unwrap()),
                "No targets found",
            );
        } else {
            let ntotal = ys
                .values()
                .fold(0, |nx, x| nx + x.values().fold(0, |ny, y| ny + y.len()));
            self.status_log(
                true,
                "Found",
                &format!("{:?} x{}", self.target.unwrap(), ntotal),
                "",
            );

            // check method
            self.ask_method(theme)?;

            // question asking according to different methods
            match self.method.unwrap() {
                Method::Random => self.ask_nbit(theme, ntotal)?,
                Method::Znum => {
                    self.ask_start_from(theme)?;
                    self.ask_nbit(theme, ntotal + self.start.unwrap())?;
                }
                Method::Num => self.ask_start_from(theme)?,
                Method::Time => self.ask_delimiter(theme)?,
                Method::Prefix | Method::Append => {
                    self.ask_delimiter(theme)?;
                    self.ask_with(theme)?;
                }
                _ => {}
            }

            if !self.yes
                && !dialoguer::Confirm::with_theme(theme)
                    .with_prompt("Ready to go")
                    .default(true)
                    .show_default(true)
                    .wait_for_newline(true)
                    .interact()?
            {
                self.status_log(false, "Task cancelled.", "", "");
            }
            let pb = self.build_progressbar(ntotal as u64, " Renaming".to_string());
            let mut map_pf_stem = HashMap::<PathBuf, String>::new();
            let mut map_pd_cnt: HashMap<PathBuf, usize> = HashMap::new();
            let mut f_cache = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&self.cache)?;

            // loop
            for (_, paths) in ys.iter().rev() {
                for (pd, pfs) in paths.iter() {
                    for pf in pfs.iter() {
                        pb.inc(1);
                        let path_new = self.gen_uniq(pf, pd, &mut map_pd_cnt, &mut map_pf_stem);
                        self.rename_and_cache(pf, &path_new, &mut f_cache)?;
                    }
                }
            }
            pb.finish();
        }
        Ok(())
    }

    pub fn roll_back(&self) -> Result<()> {
        if !std::path::Path::new(&self.cache).exists() {
            self.status_log(
                false,
                "Execution cache",
                "Not Found",
                "Please run renify beforehand",
            );
        }
        let contents = std::fs::read_to_string(&self.cache)?;
        let pb =
            self.build_progressbar(contents.lines().count() as u64, " Rolling Back".to_string());
        let mut f_cache = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cache)?;
        for line in contents.lines().rev() {
            pb.inc(1);
            let v: Vec<&str> = line.split(' ').collect();
            self.rename_and_cache(v[1], v[0], &mut f_cache)?;
        }
        pb.finish();
        Ok(())
    }

    fn fetch_targets<P: AsRef<Path>>(
        &self,
        source: P,
    ) -> Result<BTreeMap<usize, BTreeMap<PathBuf, Vec<PathBuf>>>> {
        let source = source.as_ref().canonicalize().unwrap();
        let mut ys = if source.is_file() {
            let mut ys = BTreeMap::new();
            let mut y = BTreeMap::new();
            y.insert(
                source
                    .parent()
                    .expect("You can not reach the parent of root directory.")
                    .to_path_buf(),
                vec![source.to_path_buf()],
            );
            ys.insert(0_usize, y);
            ys
        } else {
            let mut ys: BTreeMap<usize, BTreeMap<PathBuf, Vec<PathBuf>>> = BTreeMap::new();
            for entry in WalkDir::new(source)
                .follow_links(matches!(self.target.unwrap(), Target::Symlink))
                .into_iter()
                .filter_entry(|x| !self._is_hidden(x))
            {
                match entry {
                    Ok(entry) => {
                        // skip root dir
                        if entry.file_type().is_dir() && entry.depth() == 0 {
                            continue;
                        }

                        // hidden excluded
                        // if !self.hidden_included && self._is_hidden(&entry) {
                        //     continue;
                        // }

                        // non-recrusive
                        if !self.recursive.unwrap() && entry.depth() > 1 {
                            continue;
                        }

                        // depth
                        if let Some(d) = self.depth {
                            if entry.depth() > d {
                                continue;
                            }
                        }

                        // symlinks are not included!
                        // TODO: files or dirs that links(soft & hard) points to are not included!
                        if entry.path_is_symlink() {
                            continue;
                        }

                        // filter
                        if entry.file_type().is_file() {
                            if let Target::File = self.target.unwrap() {
                            } else {
                                continue;
                            }
                        } else if entry.file_type().is_dir() {
                            if let Target::Dir = self.target.unwrap() {
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        // save
                        ys.entry(entry.depth())
                            .or_default()
                            .entry(
                                entry
                                    .path()
                                    .parent()
                                    .expect("You can not reach the parent of root directory.")
                                    .to_path_buf(),
                            )
                            .or_default()
                            .push(entry.path().to_path_buf());
                    }
                    Err(e) => {
                        println!("walkdir error: {:?} (Basically won't happen)", e);
                    }
                }
            }
            ys
        };

        // re-order
        for (_, paths) in ys.iter_mut() {
            for (_, path) in paths.iter_mut() {
                path.sort_by(|a, b| {
                    a.file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap_or(usize::MAX)
                        .cmp(
                            &b.file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .parse::<usize>()
                                .unwrap_or(usize::MAX),
                        )
                });
            }
        }
        Ok(ys)
    }

    pub fn gen_uniq(
        &self,
        pf: &Path,
        pd: &Path,
        map_pd_cnt: &mut HashMap<PathBuf, usize>,
        map_pf_stem: &mut HashMap<PathBuf, String>,
    ) -> PathBuf {
        // generate unique file stem
        if !self.indiscriminate {
            if let Target::File = self.target.unwrap() {
                let path_wo_ext = pf.with_extension("");
                if !map_pf_stem.is_empty() && map_pf_stem.contains_key(&path_wo_ext) {
                    let _stem = &map_pf_stem[&path_wo_ext];
                    let mut path_new = pf.with_file_name(_stem);
                    if let Some(suffix) = pf.extension() {
                        path_new = path_new.with_extension(suffix);
                    }
                    return path_new;
                }
            }
        }

        loop {
            let stem = match self.method.unwrap() {
                Method::Time => chrono::Local::now()
                    .format(&format!(
                        "%Y{}%m{}%d{}%H{}%M{}%S{}%f",
                        self.delimiter.as_ref().unwrap(),
                        self.delimiter.as_ref().unwrap(),
                        self.delimiter.as_ref().unwrap(),
                        self.delimiter.as_ref().unwrap(),
                        self.delimiter.as_ref().unwrap(),
                        self.delimiter.as_ref().unwrap()
                    ))
                    .to_string(),
                Method::Uuid => Uuid::new_v4().to_string(),
                Method::Prefix => {
                    assert!(
                            self.with.is_some(),
                            "> You should set the prefix content by `--with` when using `Method::Prefix`."
                        );
                    let stem = pf.file_stem().unwrap().to_str().unwrap();
                    format!(
                        "{}{}{}",
                        self.with.clone().unwrap(),
                        self.delimiter.as_ref().unwrap(),
                        stem
                    )
                }
                Method::Append => {
                    assert!(
                            self.with.is_some(),
                            "> You should set the postfix content by `--with` when using `Method::Append`."
                        );
                    let stem = pf.file_stem().unwrap().to_str().unwrap();
                    format!(
                        "{}{}{}",
                        stem,
                        self.delimiter.as_ref().unwrap(),
                        self.with.clone().unwrap()
                    )
                }
                Method::Num | Method::Znum => {
                    let count = map_pd_cnt
                        .entry(pd.to_path_buf())
                        .or_insert(self.start.unwrap() - 1);
                    *count += 1;
                    if let Method::Znum = self.method.unwrap() {
                        format!("{:0>1$}", count, self.nbits.unwrap())
                    } else {
                        count.to_string()
                    }
                }
                Method::Random => thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(self.nbits.unwrap())
                    .map(char::from)
                    .collect(),
                // Method::Lowercase => {
                //     let stem = pf.file_stem().unwrap().to_str().unwrap().to_lowercase();
                //     stem
                // }
                // Method::Uppercase => {
                //     let stem = pf.file_stem().unwrap().to_str().unwrap().to_uppercase();
                //     stem
                // }
                _ => todo!(),
            };

            // check if new stem file exists
            let mut p_new = pf.with_file_name(stem);

            // extend with suffix
            if let Some(suffix) = pf.extension() {
                p_new = p_new.with_extension(suffix);
            }

            // save if keep consistance
            if !self.indiscriminate {
                if let Target::File = self.target.unwrap() {
                    let _stem = p_new.file_stem().unwrap().to_str().unwrap().to_string();
                    let path_wo_ext = pf.with_extension("");
                    map_pf_stem.insert(path_wo_ext, _stem);
                }
            }

            if !p_new.exists() {
                break p_new;
            } else {
                match self.method.unwrap() {
                    // Method::Uppercase | Method::Lowercase | Method::Num | Method::Znum => {
                    Method::Num | Method::Znum => {
                        break pf.to_path_buf();
                    }
                    _ => {}
                }
            }
        }
    }

    fn _is_hidden(&self, entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|p| p.starts_with('.'))
            .unwrap_or(false)
    }

    fn max_depth<P: AsRef<Path>>(&self, source: P) -> usize {
        let source = source.as_ref();
        let mut depth = 0usize;
        for entry in WalkDir::new(source).into_iter() {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_dir() {
                        if let Target::Dir = self.target.unwrap() {
                            depth = entry.depth().max(depth);
                        } else {
                            continue;
                        }
                    } else if entry.file_type().is_file() {
                        if let Target::File = self.target.unwrap() {
                            depth = entry.depth().max(depth);
                        } else {
                            continue;
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        depth
    }

    fn rename_and_cache<P: AsRef<Path>>(&self, p0: P, p1: P, f: &mut std::fs::File) -> Result<()> {
        let p0 = p0.as_ref().canonicalize()?;
        std::fs::rename(&p0, &p1)?;
        let p1 = p1.as_ref().canonicalize()?;
        let _map = format!("{} {}\n", p0.display(), p1.display());
        f.write_all(_map.as_bytes())?;
        Ok(())
    }

    #[allow(clippy::println_empty_string)]
    fn status_log(&self, status: bool, t1: &str, t2: &str, prompt: &str) {
        if status {
            print!(
                "{}",
                console::Style::new()
                    .bold()
                    .color256(49)
                    .bright()
                    .apply_to("✔  ")
            );
        } else {
            print!(
                "{}",
                console::Style::new()
                    .bold()
                    .color256(9)
                    .bright()
                    .apply_to("✘  ")
            );
        }

        // t1
        print!(
            "{}",
            console::Style::new().white().bold().bright().apply_to(t1)
        );

        // t2
        if !t2.is_empty() {
            print!(
                "{}{}",
                console::Style::new().bold().white().dim().apply_to(" · "),
                console::Style::new().color256(49).bright().apply_to(t2),
            );
        }

        // prompt
        if !prompt.is_empty() {
            print!(
                "{}{}",
                console::Style::new().black().bright().apply_to(" › "),
                console::Style::new()
                    .black()
                    .bright()
                    .apply_to(format!(" {}", prompt)),
            );
        }
        println!("");

        if !status {
            std::process::exit(0);
        }
    }

    fn check_source(&self) -> Result<Target> {
        let p = std::path::Path::new(&self.input);
        if !p.exists() {
            self.status_log(false, "Source", " Not Exist", &self.input);
        }
        // check type
        let type_ = if p.is_symlink() {
            Target::Symlink
        } else if p.is_file() {
            Target::File
        } else {
            Target::Dir
        };
        let p = p.canonicalize()?;
        let p = p.to_str().unwrap();
        self.status_log(
            true,
            "Source",
            match type_ {
                Target::Dir | Target::File => p,
                Target::Symlink => &self.input,
            },
            match type_ {
                Target::Dir => "A Folder",
                Target::File => "A File",
                Target::Symlink => "A Symlink",
            },
        );
        Ok(type_)
    }

    fn ask_target(&mut self, source_type: Target, theme: &ColorfulTheme) -> Result<()> {
        match source_type {
            Target::Symlink => {
                self.status_log(false, "Source Error", "Not Supported", "Symlink");
            }
            Target::File => {
                self.target = Some(Target::File);
                self.status_log(true, "Target", "File", "--target file");
            }
            Target::Dir => match self.target {
                None => {
                    let selections = ["File", "Folder"];
                    let i = dialoguer::Select::with_theme(theme)
                        .with_prompt("Target")
                        .default(0)
                        .max_length(3)
                        .items(&selections[..])
                        .interact()?;
                    self.target = Some(Target::from(selections[i]));
                }
                Some(target) => {
                    self.status_log(
                        true,
                        "Target",
                        &format!("{:?}", target),
                        &format!(
                            "--target {}",
                            match target {
                                Target::Dir => "dir",
                                Target::File => "file",
                                _ => "",
                            }
                        ),
                    );
                }
            },
        }

        // recursive & depth
        if let Target::Dir = source_type {
            self.ask_recursive(theme)?;
        }

        // file stem consistent
        if let Target::File = self.target.unwrap() {
            self.status_log(
                true,
                "Preserve name consistent",
                &format!("{:?}", !self.indiscriminate),
                &format!("--indiscriminate {}", self.indiscriminate),
            );
        }

        Ok(())
    }

    fn ask_method(&mut self, theme: &ColorfulTheme) -> Result<()> {
        match self.method {
            None => {
                let selections = &[
                    "Random",
                    "Uuid",
                    "Time",
                    "Numbered",
                    "Zero-Numbered",
                    "Prefix",
                    "Append",
                    // "Lowercase",
                    // "Uppercase",
                ];
                let i = dialoguer::Select::with_theme(theme)
                    .with_prompt("Select method")
                    .default(0)
                    .max_length(10)
                    .items(&selections[..])
                    .interact()?;
                self.method = Some(Method::from(selections[i]));
            }
            Some(method) => {
                self.status_log(
                    true,
                    "Method",
                    &format!("{:?}", method),
                    &format!(
                        "--method {}",
                        match method {
                            Method::Random => "random",
                            Method::Time => "time",
                            Method::Num => "num",
                            Method::Znum => "znum",
                            Method::Prefix => "prefix",
                            Method::Append => "append",
                            Method::Uuid => "uuid",
                            _ => "",
                        }
                    ),
                );
            }
        }
        Ok(())
    }

    fn ask_recursive(&mut self, theme: &ColorfulTheme) -> Result<()> {
        match self.recursive {
            None => {
                self.recursive = Some(
                    dialoguer::Confirm::with_theme(theme)
                        .with_prompt("Doing recursively")
                        .default(false)
                        .show_default(true)
                        .wait_for_newline(true)
                        .interact()?,
                );
            }
            Some(recursive) => {
                self.status_log(
                    true,
                    "Recursively",
                    &format!("{:?}", recursive),
                    &format!("--recursive {}", self.recursive.unwrap()),
                );
            }
        }

        // folder depth
        if self.recursive.unwrap() {
            self.ask_depth(theme)?;
        }
        Ok(())
    }

    fn ask_start_from(&mut self, theme: &ColorfulTheme) -> Result<()> {
        match self.start {
            None => {
                self.start = Some(
                    dialoguer::Input::with_theme(theme)
                        .with_prompt("Start from")
                        .with_initial_text("1".to_string())
                        .validate_with(|input: &String| -> Result<(), &str> {
                            match input.parse::<usize>() {
                                Ok(_) => Ok(()),
                                Err(_) => Err("It should be a number and be greater than 0!"),
                            }
                        })
                        .allow_empty(false)
                        .interact_text()?
                        .parse::<usize>()?,
                );
            }
            Some(n) => {
                self.status_log(
                    true,
                    "Number start from",
                    &format!("{:?}", n),
                    &format!("--start {}", n),
                );
            }
        }

        Ok(())
    }

    fn decimal_to_62(decimal: usize) -> String {
        let characters: Vec<char> =
            "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
                .chars()
                .collect();
        let base: usize = 62;
        let mut num = decimal;
        if num == 0 {
            return "0".to_string();
        }
        let mut result = String::new();
        while num > 0 {
            result.push(characters[num % base]);
            num /= base;
        }
        result.chars().rev().collect()
    }

    fn ask_nbit(&mut self, theme: &ColorfulTheme, ntotal: usize) -> Result<()> {
        //  calculate the bit_min
        let n_min = match self.method.unwrap() {
            Method::Znum => ntotal.to_string().len(),
            Method::Random => Self::decimal_to_62(ntotal).len(),
            _ => 0,
        };
        let n_max = n_min + 5;
        let err_msg = format!("It should be between {} to {}.", n_min, n_max);
        match self.nbits {
            None => {
                self.nbits = Some(
                    dialoguer::Input::with_theme(theme)
                        .with_prompt("The number of bits")
                        .with_initial_text(n_min.to_string())
                        .validate_with(|input: &String| -> Result<(), &str> {
                            match input.parse::<usize>() {
                                Ok(n) => {
                                    if !(n_min..=n_max).contains(&n) {
                                        Err(&err_msg)
                                    } else {
                                        Ok(())
                                    }
                                }
                                Err(_) => Err("This is not a number!"),
                            }
                        })
                        .allow_empty(false)
                        .interact_text()?
                        .parse::<usize>()?,
                );
            }
            Some(n) => {
                // validate
                if !(n_min..=n_max).contains(&n) {
                    self.status_log(false, "The number of bits", &n.to_string(), &err_msg);
                }
                self.status_log(
                    true,
                    "The number of bits",
                    &format!("{:?}", n),
                    &format!("--nbits {}", n),
                );
            }
        }

        Ok(())
    }

    fn ask_depth(&mut self, theme: &ColorfulTheme) -> Result<()> {
        let depth_max = self.max_depth(&self.input);
        match self.depth {
            None => {
                let depth = dialoguer::Input::with_theme(theme)
                    .with_prompt(format!(
                        "The depth of folder (max: {}, 0: to the end)",
                        depth_max
                    ))
                    .with_initial_text("0".to_string())
                    .validate_with(|input: &String| -> Result<(), &str> {
                        match input.parse::<usize>() {
                            Ok(n) => {
                                if !(0..=depth_max).contains(&n) {
                                    Err("It should be between 0 to max depth found.")
                                } else {
                                    Ok(())
                                }
                            }
                            Err(_) => Err("This is not a number!"),
                        }
                    })
                    .allow_empty(false)
                    .interact_text()?
                    .parse::<usize>()?;
                self.depth = if depth == 0usize { None } else { Some(depth) };
            }
            Some(depth) => {
                self.status_log(
                    true,
                    "The depth of folder",
                    &format!("{:?}", depth),
                    &format!("--depth {}", depth),
                );
            }
        }
        Ok(())
    }

    fn ask_delimiter(&mut self, theme: &ColorfulTheme) -> Result<()> {
        let err_msg = format!(
            "Wrong char! These are usually not allowed: {}",
            STR_NOT_ALLOWED
        );
        match &self.delimiter {
            None => {
                self.delimiter = Some(
                    dialoguer::Input::with_theme(theme)
                        .with_prompt("Delimiter")
                        .with_initial_text("-".to_string())
                        .validate_with({
                            |input: &String| -> Result<(), &str> {
                                if input.as_str().chars().any(|c| STR_NOT_ALLOWED.contains(c)) {
                                    Err(&err_msg)
                                } else {
                                    Ok(())
                                }
                            }
                        })
                        .allow_empty(true)
                        .interact_text()?,
                )
            }
            Some(delimiter) => {
                self.status_log(
                    true,
                    "Delimiter",
                    delimiter,
                    &format!("--delimiter {}", delimiter),
                );
            }
        }

        Ok(())
    }

    fn ask_with(&mut self, theme: &ColorfulTheme) -> Result<()> {
        let err_msg = format!(
            "Wrong char! These are usually not allowed: {}",
            STR_NOT_ALLOWED
        );
        match &self.with {
            None => {
                self.with = Some(
                    dialoguer::Input::with_theme(theme)
                        .with_prompt("What text with")
                        .validate_with({
                            |input: &String| -> Result<(), &str> {
                                if input.as_str().chars().any(|c| STR_NOT_ALLOWED.contains(c)) {
                                    Err(&err_msg)
                                } else {
                                    Ok(())
                                }
                            }
                        })
                        .allow_empty(false)
                        .interact_text()?,
                );
            }
            Some(with) => {
                self.status_log(true, "What text with", with, &format!("--with {}", with));
            }
        }

        Ok(())
    }

    fn build_theme() -> ColorfulTheme {
        ColorfulTheme {
            // palette: Color256(9), Color256(49)
            defaults_style: console::Style::new().bold().color256(49),
            hint_style: console::Style::new().white().dim(),
            values_style: console::Style::new().color256(49).bright(),
            prompt_style: console::Style::new().white().bold().bright(),
            prompt_prefix: console::Style::new()
                .bold()
                .color256(9)
                .apply_to("? ".to_string()),
            prompt_suffix: console::Style::new().white().dim().apply_to("› ".into()),
            success_prefix: console::Style::new()
                .bold()
                .color256(49)
                .bright()
                .apply_to("✔ ".to_string()),
            success_suffix: console::Style::new()
                .bold()
                .white()
                .dim()
                .apply_to("·".to_string()),
            error_prefix: console::Style::new()
                .bold()
                .color256(9)
                .bright()
                .apply_to("✘ ".to_string()),
            error_style: console::Style::new().color256(9).bold().bright(),
            active_item_prefix: console::Style::new()
                .bold()
                .apply_to(console::Emoji("👉", "").to_string()),
            inactive_item_prefix: console::Style::new()
                .white()
                .dim()
                .apply_to("  ".to_string()),
            active_item_style: console::Style::new().bold().color256(49).bright(),
            inactive_item_style: console::Style::new().white().dim(),
            ..Default::default()
        }
    }

    fn build_progressbar(&self, size: u64, prefix: String) -> ProgressBar {
        let pb = ProgressBar::new(size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green.bold} {prefix:.bold} [{bar:.magenta.bright.bold/white.dim}] {human_pos}/{human_len} ({percent}% | {eta} | {elapsed_precise})"
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write | write!(w, "{:.2}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        pb.set_prefix(prefix);
        pb
    }
}

use std::{
    cell::RefCell,
    fs::{self, File},
    ops::Deref,
    path::Path,
    process::ExitStatus,
    rc::Rc,
};

use clap::Args;

#[derive(Debug, Args, Clone)]
pub struct ArchiveArg {
    #[clap(short = 's')]
    pub artifacts_dir: String,

    #[clap(
        short = 'd',
        env = "UPROG_PATH",
        default_value_t =
            std::env::current_dir().unwrap()
                .join("uprog.jrz")
                .to_str()
                .unwrap()
                .to_string()
    )]
    pub archive_path: String,
}

#[must_use]
pub fn run(arg: &ArchiveArg) -> Option<ExitStatus> {
    let ArchiveArg {
        ref artifacts_dir,
        ref archive_path,
        ..
    } = arg.clone();

    let artifacts_dir = fs::canonicalize(artifacts_dir).unwrap();
    let file = File::create(archive_path).unwrap();
    let input = Rc::new(RefCell::new(Vec::new()));

    archive_dir_all(&artifacts_dir, &artifacts_dir, input.clone());
    cpio::write_cpio(input.to_owned().take().into_iter(), file).unwrap();

    Some(ExitStatus::default())
}

fn archive_dir_all(prefix: &Path, dir: &Path, output: Rc<RefCell<Vec<(cpio::NewcBuilder, File)>>>) {
    for entry in fs::read_dir(dir).unwrap() {
        let output = output.clone();
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            archive_dir_all(prefix, &path, output);
        } else {
            let slug = path.strip_prefix(prefix).unwrap().to_str().unwrap();
            let builder = cpio::NewcBuilder::new(slug);
            let file = fs::File::open(&path).unwrap();
            output.deref().borrow_mut().push((builder, file));
        }
    }
}

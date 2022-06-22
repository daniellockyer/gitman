use clap::Parser;
use git2::{Repository, StatusOptions};
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directory to recurse into
    #[clap(short, long, value_parser, default_value_t = std::env::current_dir().expect("Could not get CWD").into_os_string().into_string().expect("Could not convert to string"))]
    cwd: String,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_git_repo(entry: &DirEntry) -> bool {
    entry.path().is_dir() && {
        let p = entry.path().join(".git").join("config");
        p.exists() && p.is_file()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut found = Vec::new();

    let mut it = WalkDir::new(args.cwd).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(err)) => {
                eprintln!("{:?}", err);
                continue;
            }
            Some(Ok(entry)) => entry,
        };

        if is_hidden(&entry) {
            if entry.file_type().is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        if is_git_repo(&entry) {
            found.push(entry.clone());
            it.skip_current_dir();
            continue;
        }
    }

    let mut changes_count = 0;

    for entry in &found {
        if let Ok(repo) = Repository::open(entry.path()) {
            let mut opts = StatusOptions::new();
            opts.include_untracked(true);
            let statuses = repo
                .statuses(Some(&mut opts))
                .map_err(|e| format!("{}", e))?;

            for status in statuses.iter() {
                if let Some(path) = status.path() {
                    eprintln!(
                        "{}/{} - {:?}",
                        entry.path().display(),
                        path,
                        status.status()
                    );
                    changes_count += 1;
                }
            }
        }
    }

    println!("{} repos found, {} changes", found.len(), changes_count);

    Ok(())
}

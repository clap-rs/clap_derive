#[test]
fn top_level_enum() {
    #[derive(ClapApp)]
    #[clap(name = "git", about = "the stupid content tracker")]
    enum Git {
        #[derive(Debug)]
        #[clap(name = "add")]
        Add {
            #[clap(short = "i")]
            interactive: bool,
            #[clap(short = "p")]
            patch: bool,
            files: Vec<String>
        },
        #[derive(Debug)]
        #[clap(name = "fetch")]
        Fetch {
            #[clap(long = "dry-run")]
            dry_run: bool,
            #[clap(long = "all")]
            all: bool,
            repository: Option<String>
        },
        #[derive(Debug)]
        #[clap(name = "commit")]
        Commit {
            #[clap(short = "m")]
            message: Option<String>,
            #[clap(short = "a")]
            all: bool
        }
    }

    let git = Git::parse_from(vec!["git", "add", "-i", "file1", "file2"]);
    match git {
        add @ Add { .. } => {
            assert!(!add.patch);
            assert!(add.interactive);
            assert_eq!(add.files, &["file1", "file2"]);
        },
        _ => panic!("Wrong subcommand was found in tests/subcommands.rs:top_level_enum");
    }
}

#[test]
fn top_level_struct() {
    #[derive(ClapApp)]
    #[clap(name = "git")]
    struct Git {
        #[clap(long = "verbose")]
        verbose: bool,
        #[clap(subcommand)]  
        cmd: GitCmd
    }

    #[derive(ClapSubCommands)]
    enum GitCmd {
        #[derive(Debug)]
        #[clap(name = "add")]
        /// Add changes to be commited
        Add {
            #[clap(short = "i")]
            interactive: bool,
            #[clap(short = "p")]
            patch: bool,
            files: Vec<String>
        },
        #[derive(Debug)]
        #[clap(name = "fetch")]
        /// Fetch remote repos
        Fetch {
            #[clap(long = "dry-run")]
            dry_run: bool,
            #[clap(long = "all")]
            all: bool,
        },
        #[derive(Debug)]
        #[clap(name = "commit")]
        /// Commit changes
        Commit {
            #[clap(short = "m")]
            message: Option<String>,
            #[clap(short = "a")]
            all: bool
        }
    }

    let git = Git::parse_from(vec!["git", "add", "-i", "file1", "file2"]);
    match git {
        add @ Add { .. } => {
            assert!(!add.patch);
            assert!(add.interactive);
            assert_eq!(add.files, &["file1", "file2"]);
        },
        _ => panic!("Wrong subcommand was found in tests/subcommands.rs:top_level_enum");
    }
}

#[test]
fn top_level_struct_no_subcmd_provided_fail() {
    #[derive(ClapApp)]
    #[clap(name = "git")]
    struct Git {
        #[clap(long = "verbose")]
        verbose: bool,
        #[clap(subcommand)]  
        cmd: GitCmd
    }

    #[derive(ClapSubCommands)]
    enum GitCmd {
        #[derive(Debug)]
        #[clap(name = "add")]
        /// Add changes to be commited
        Add {
            #[clap(short = "i")]
            interactive: bool,
            #[clap(short = "p")]
            patch: bool,
            files: Vec<String>
        },
        #[derive(Debug)]
        #[clap(name = "fetch")]
        /// Fetch remote repos
        Fetch {
            #[clap(long = "dry-run")]
            dry_run: bool,
            #[clap(long = "all")]
            all: bool,
        },
        #[derive(Debug)]
        #[clap(name = "commit")]
        /// Commit changes
        Commit {
            #[clap(short = "m")]
            message: Option<String>,
            #[clap(short = "a")]
            all: bool
        }
    }

    let res = Git::try_parse_from(vec!["git"]);
    assert!(res.is_err());
    // need to check error kind
}

#[test]
fn top_level_struct_nested_subcmds() {
    #[derive(ClapApp)]
    #[clap(name = "git")]
    struct Git {
        #[clap(long = "verbose")]
        verbose: bool,
        #[clap(subcommand)]  
        cmd: GitCmd
    }

    #[derive(ClapSubCommands)]
    enum GitCmd {
        #[derive(Debug)]
        #[clap(name = "add")]
        /// Add changes to be commited
        Add {
            #[clap(short = "i")]
            interactive: bool,
            #[clap(short = "p")]
            patch: bool,
            files: Vec<String>
        },
        #[derive(Debug)]
        #[clap(name = "fetch")]
        /// Fetch remote repos
        Fetch {
            #[clap(long = "dry-run")]
            dry_run: bool,
            #[clap(long = "all")]
            all: bool,
            #[clap(subcommand)]
            repo_cmd: Option<FetchCmd>
        },
        #[derive(Debug)]
        #[clap(name = "commit")]
        /// Commit changes
        Commit {
            #[clap(short = "m")]
            message: Option<String>,
            #[clap(short = "a")]
            all: bool
        }
    }

    #[derive(ClapSubCommands)]
    enum FetchCmd {
        #[clap(name = "remote")]
        /// Handle fetching remote repos
        Remote {
            url: String
        },
        #[clap(name = "local")]
        /// Handle fetching local repos
        Local {
            #[default_value = "."]
            dir: String,
        }
    }

    let git = Git::parse_from(vec!["git", "-v", "fetch", "--all", "remote", "someurl"]);
    assert!(git.verbose);
    match git.cmd {
        fetch @ Fetch { .. } => {
            assert!(fetch.all);
            match fetch.repo_cmd {
                remote @ Remote { .. } => {
                    assert_eq!(remote.url, "someurl");
                },
                _ => panic!("Wrong subcommand was found in tests/subcommands.rs:top_level_enum");
            }
        },
        _ => panic!("Wrong subcommand was found in tests/subcommands.rs:top_level_enum");
    }
}

#[test]
fn all_separate_structs() {
    #[derive(ClapApp)]
    #[clap(name = "git")]
    struct Git {
        #[clap(long = "verbose")]
        verbose: bool,
        #[clap(subcommand)]  // Note that we mark a field as a subcommand
        cmd: GitCmd
    }

    #[derive(ClapSubCommands)]
    enum GitCmd {
        Add(Add),
        Fetch(Fetch),
        Commit(Commmit)
    }

    #[derive(Debug)]
    #[clap(name = "add")]
    /// Add changes to be commited
    Add {
        #[clap(short = "i")]
        interactive: bool,
        #[clap(short = "p")]
        patch: bool,
        files: Vec<String>
    },
    #[derive(Debug)]
    #[clap(name = "fetch")]
    /// Fetch remote repos
    Fetch {
        #[clap(long = "dry-run")]
        dry_run: bool,
        #[clap(long = "all")]
        all: bool,
        #[clap(subcommand)]
        repository: Option<FetchCmd>
    },
    #[derive(Debug)]
    #[clap(name = "commit")]
    /// Commit changes
    Commit {
        #[clap(short = "m")]
        message: Option<String>,
        #[clap(short = "a")]
        all: bool
    }

    #[derive(ClapSubCommands)]
    enum FetchCmd {
        Remote(Remote),
        Local(Local)
    }

    #[clap(name = "remote")]
    /// Handle fetching remote repos
    Remote {
        url: String
    },
    #[clap(name = "local")]
    /// Handle fetching local repos
    Local {
        #[default_value = "."]
        dir: String,
    }

    let git = Git::parse_from(vec!["git", "-v", "fetch", "--all", "remote", "someurl"]);
    assert!(git.verbose);
    match git.cmd {
        fetch @ Fetch { .. } => {
            assert!(fetch.all);
            match fetch.repo_cmd {
                remote @ Remote { .. } => {
                    assert_eq!(remote.url, "someurl");
                },
                _ => panic!("Wrong subcommand was found in tests/subcommands.rs:top_level_enum");
            }
        },
        _ => panic!("Wrong subcommand was found in tests/subcommands.rs:top_level_enum");
    }
}
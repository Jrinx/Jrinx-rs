use alloc::{borrow::ToOwned, string::String, vec::Vec};
use getargs::{Opt, Options};

use crate::{task, util::once_lock::OnceLock};

static BOOTARGS: OnceLock<String> = OnceLock::new();

pub(super) fn set(bootargs: &str) {
    BOOTARGS.init(bootargs.to_owned()).unwrap();
}

pub async fn execute() {
    if let Some(bootargs) = BOOTARGS.get() {
        let args = bootargs
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        let mut opts = Options::new(args.iter().map(String::as_str));

        info!("bootargs: {}", bootargs);

        while let Some(opt) = opts.next_opt().unwrap() {
            match opt {
                Opt::Short('h') | Opt::Long("help") => {
                    info!("boot arguments:");
                    info!("   -t, --test <test>    Run the specified test");
                    info!("   -h, --help           Display this information");
                }

                Opt::Short('t') | Opt::Long("test") => {
                    let arg = match opts.value() {
                        Ok(opt) => opt,
                        _ => {
                            panic!("missing argument for option: {opt}, try '-t/--test help' for more information");
                        }
                    };
                    if arg == "help" {
                        info!("all available tests:");
                        let mut all_tests = jrinx_testdef::all().collect::<Vec<_>>();
                        all_tests.sort();
                        all_tests.iter().for_each(|test| info!("- {test}"));
                    } else {
                        let test = arg;
                        let func = jrinx_testdef::find(test)
                            .unwrap_or_else(|| panic!("unrecognized test case: {}", test));
                        info!("test case {} begin", test);
                        task::spawn!(async move {
                            func();
                        });
                        task::yield_now!();
                        info!("test case {} end", test);
                    }
                }

                Opt::Short(_) | Opt::Long(_) => panic!("unrecognized option: {}", opt),
            };
        }
    }
}

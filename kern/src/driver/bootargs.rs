use alloc::{borrow::ToOwned, format, string::String, vec::Vec};
use getargs::{Opt, Options};

use crate::{task, test};

static mut BOOTARGS: Option<String> = None;

pub(super) fn set(bootargs: &str) {
    unsafe {
        BOOTARGS = Some(bootargs.to_owned());
    }
}

pub async fn execute() {
    if let Some(bootargs) = unsafe { &mut BOOTARGS } {
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
                        let mut all_tests = test::all().collect::<Vec<_>>();
                        all_tests.sort();
                        all_tests.iter().for_each(|test| info!("- {test}"));
                    } else {
                        let test = arg;
                        let func = test::find(test)
                            .expect(format!("unrecognized test case: {}", test).as_str());
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

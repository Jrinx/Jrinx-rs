use alloc::{borrow::ToOwned, collections::BTreeMap, format, string::String, sync::Arc, vec::Vec};
use core::num::ParseIntError;

use getargs::{Opt, Options};
use jrinx_a653::{
    partition::{Partition, PartitionConfig, PartitionId, PartitionTypeConfig},
    process::{Process, ProcessRunner},
};
use jrinx_apex::*;
use jrinx_hal::{Cpu, Hal};
use jrinx_multitask::{
    inspector::Inspector,
    runtime::{Runtime, RuntimeSchedTable, RuntimeSchedTableEntry},
    spawn, yield_now,
};
use spin::Once;

static BOOTARGS: Once<String> = Once::new();

pub(super) fn set(bootargs: &str) {
    BOOTARGS
        .try_call_once::<_, ()>(|| Ok(bootargs.to_owned()))
        .unwrap();
}

pub async fn execute() {
    if let Some(bootargs) = BOOTARGS.get() {
        let args = bootargs
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        let mut opts = Options::new(args.iter().map(String::as_str));

        info!("bootargs: {}", bootargs.replace("--", "\n\t--"));

        let mut partitions: Vec<Arc<Partition>> = Vec::new();

        while let Some(opt) = opts.next_opt().unwrap() {
            match opt {
                Opt::Short('h') | Opt::Long("help") => help().await,

                Opt::Short('t') | Opt::Long("test") => {
                    test(match opts.value() {
                        Ok(opt) => opt,
                        _ => {
                            panic!("missing argument for option: {opt}, try '-t/--test help' for more information");
                        }
                    }).await;
                }

                Opt::Long("partition") => {
                    if let Some(partition) = partition(match opts.value() {
                        Ok(opt) => opt,
                        _ => {
                            panic!("missing argument for option: {opt}, try '--partition help' for more information");
                        }
                    }).await {
                        partitions.push(partition);
                    }
                }

                Opt::Long("scheduler") => {
                    if let Some((cpu_id, sched_table, inspectors)) = scheduler(match opts.value() {
                        Ok(opt) => opt,
                        _ => {
                            panic!("missing argument for option: {opt}, try '--scheduler help' for more information");
                        }
                    }, &partitions).await {
                        Runtime::with_spec_cpu(cpu_id, |rt| {
                            for inspector in inspectors {
                                rt.register(inspector).unwrap();
                            }
                            rt.enact_sched_table(sched_table).unwrap();
                        }).unwrap();
                    }
                }

                Opt::Short(_) | Opt::Long(_) => panic!("unrecognized option: {}", opt),
            };
        }
    }
}

async fn help() {
    info!("boot arguments:");
    info!("       --partition <opts>  Create a partition");
    info!("                           * use '--partition help' for more information");
    info!("       --scheduler <opts>  Create a scheduler to schedule partitions");
    info!("                           * use '--scheduler help' for more information");
    info!("   -t, --test <test>       Run the specified test");
    info!("   -h, --help              Display this information");
}

async fn test(args: &str) {
    if args == "help" {
        info!("all available tests:");
        let mut all_tests = jrinx_testdef::all().collect::<Vec<_>>();
        all_tests.sort();
        all_tests.iter().for_each(|test| info!("- {test}"));
    } else {
        let test = args;
        let (name, func) =
            jrinx_testdef::find(test).unwrap_or_else(|| panic!("unrecognized test case: {}", test));
        info!("test case {} begin", name);
        spawn!(async move {
            func();
        });
        yield_now!();
        info!("test case {} end", name);
    }
}

async fn partition(args: &str) -> Option<Arc<Partition>> {
    let nproc = hal!().cpu().nproc_valid();

    if args == "help" {
        info!("To create a partition, you need to specify its kern/user property and partition configuration");
        info!("Required (comma-seperated) arguments to create a partition configuration:");
        info!("   name=<str>                Specify the name of the partition");
        info!("   memory=<unsigned>         Specify the memory limit of the partition");
        info!("                             * the radix of the value is determined by the prefix");
        info!("   period=<unsigned><unit>   Specify the period of the partition");
        info!("                             * the unit can be ns, us, ms or s");
        info!("                             * the negative one indicates inf. period");
        info!("   duration=<unsigned><unit> Specify the duration of the partition");
        info!("                             * the unit can be ns, us, ms or s");
        info!("                             * the negative value indicates inf. duration");
        info!("   num_cores=<unsigned>      Specify the number of cores of the partition");
        info!("                             * up to {nproc} cores");
        info!("Required (comma-seperated) arguments to create a *kern* partition configuration:");
        info!("   entry=<str>               Specify the entry of the kernel partition (TODO)");
        info!("Required (comma-seperated) arguments to create a *user* partition configuration:");
        info!("   program=<str>             Specify the program of the user partition");
        info!("Required kern/user property to create a kern/user partition:");
        info!("   {{kern|user}}//<config>     Specify the kern/user property and partition configuration");
        info!("Example:");
        info!("   --partition user//name=example,program=idle,memory=0x8000,period=2000,duration=1000,num_cores=1");
        info!("               ^~~^  ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^");
        info!("               |     |");
        info!("               |     +-- partition configuration");
        info!("               +-------- kern/user property");
        None
    } else {
        let mut pair = args.split("//");
        let is_user = match pair.next().unwrap() {
            "kern" => false,
            "user" => true,
            _ => panic!("invalid argument: {:?}", pair),
        };

        let config = iter_key_value(pair.next().unwrap())
            .unwrap()
            .collect::<Vec<_>>();

        if pair.next().is_some() {
            panic!("invalid argument: {:?}", pair);
        }

        let name: &str = parse_key_value(config.iter(), "name").unwrap();
        let memory: usize =
            parse_usize_from_proper_redix(parse_key_value(config.iter(), "memory").unwrap())
                .unwrap();
        let period: ApexSystemTime =
            parse_time_from_proper_unit(parse_key_value(config.iter(), "period").unwrap()).unwrap();
        let duration: ApexSystemTime =
            parse_time_from_proper_unit(parse_key_value(config.iter(), "duration").unwrap())
                .unwrap();
        let num_cores: u32 = parse_key_value(config.iter(), "num_cores")
            .unwrap()
            .parse()
            .unwrap();
        let program: Option<&str> = parse_key_value(config.iter(), "program");
        if nproc < num_cores as _ {
            panic!("number of cores should be less than or equal to {nproc}, got {num_cores}");
        }
        Some(
            Partition::new(&PartitionConfig {
                name: name.try_into().unwrap(),
                memory,
                period,
                duration,
                num_cores,
                partition_type: if is_user {
                    PartitionTypeConfig::User(jrinx_uprog::find(program.unwrap()).unwrap())
                } else {
                    PartitionTypeConfig::Kern // TODO
                },
            })
            .unwrap(),
        )
    }
}

async fn scheduler(
    args: &str,
    partitions: &[Arc<Partition>],
) -> Option<(usize, RuntimeSchedTable, Vec<Inspector>)> {
    if args == "help" {
        info!("To create a scheduler, you need to specify its major-frame size, cpu-id and schedule table");
        info!("Required (comma-seperated) arguments to create a schedule table entry:");
        info!("   partition=<str>                           Specify the partition by its name");
        info!("   offset=<unsigned><unit>                   Specify the offset in the table");
        info!("                                             * the unit can be ns, us, ms or s");
        info!("   duration=<unsigned><unit>                 Specify the duration in the table");
        info!("                                             * the unit can be ns, us, ms or s");
        info!("                                             * the negative one indicates inf. duration");
        info!("                                             * the durations from entries of the same partition");
        info!("                                               should sum up to the partition's duration");
        info!("Optional (comma-seperated) arguments to create a schedule table entry:");
        info!("   init=<bool>                               Specify whether to create initial process");
        info!("                                             * default to false");
        info!("Required entries to create a schedule table:");
        info!("   <entry>;<entry>;...                       Specify the entries of the schedule table");
        info!("Required major-frame size, cpu-id and schedule table to create a scheduler:");
        info!(
              "   <unsigned><unit>#<unsigned>//<table>      Specify the major-frame size, cpu-id and schedule table"
        );
        info!("Example:");
        info!("   --scheduler 2000#0//partition=ex1,offset=0,duration=1000,init=true;partition=ex2,offset=1000,duration=1000");
        info!("               ^~~^ ^  ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^");
        info!("               |    |  |");
        info!("               |    |  +-- schedule table");
        info!("               |    +----- cpu-id");
        info!("               +---------- major-frame size");
        if !partitions.is_empty() {
            info!("Available (created) partitions:");
            partitions.iter().for_each(|partition| {
                info!("   - {}", partition.name());
            });
        }
        None
    } else {
        let mut pair = args.split('#');
        let major_frame_size: ApexSystemTime =
            parse_time_from_proper_unit(pair.next().unwrap()).unwrap();

        let mut pair = pair.next().unwrap().split("//");
        let cpu_id: usize = pair.next().unwrap().parse().unwrap();

        let entries = pair.next().unwrap().split(';');
        if pair.next().is_some() {
            panic!("invalid argument: {:?}", pair);
        }

        let mut duration_left: BTreeMap<PartitionId, ApexSystemTime> = BTreeMap::new();
        let mut table = Vec::new();
        let mut inspectors = Vec::new();
        for entry in entries {
            let config = iter_key_value(entry).unwrap().collect::<Vec<_>>();
            let partition_name: &str = parse_key_value(config.iter(), "partition").unwrap();
            let offset: ApexSystemTime =
                parse_time_from_proper_unit(parse_key_value(config.iter(), "offset").unwrap())
                    .unwrap();
            if major_frame_size != APEX_TIME_INFINITY
                && (offset == APEX_TIME_INFINITY || offset >= major_frame_size)
            {
                panic!("invalid offset: {:#?}", offset);
            }
            let duration: ApexSystemTime =
                parse_time_from_proper_unit(parse_key_value(config.iter(), "duration").unwrap())
                    .unwrap();
            let init: bool = parse_key_value(config.iter(), "init")
                .unwrap_or("false")
                .parse()
                .unwrap();

            let partition = Partition::find_by_name(&partition_name.try_into().unwrap()).unwrap();

            partition.assign_core(cpu_id as _);

            duration_left.insert(
                partition.identifier(),
                match duration_left
                    .get(&partition.identifier())
                    .unwrap_or(&partition.duration())
                {
                    left if *left == APEX_TIME_INFINITY => APEX_TIME_INFINITY,
                    left if *left < duration => {
                        panic!("invalid duration: {:#?} (left: {:#?})", duration, left)
                    }
                    left => ApexSystemTime::from(left - duration),
                },
            );

            let inspector = partition.gen_inspector().unwrap();
            if init {
                let process = Process::new_init(partition.identifier()).unwrap();
                let executor = process
                    .gen_executor(ProcessRunner {
                        syscall: jrinx_syscall::handle,
                    })
                    .unwrap();
                inspector.register(executor).unwrap();
            }
            table.push(RuntimeSchedTableEntry {
                inspector_id: inspector.id(),
                offset: time_as_duration(offset),
                period: time_as_duration(partition.period()),
                duration: time_as_duration(duration),
            });
            inspectors.push(inspector);
        }

        for (id, &left) in duration_left.iter() {
            if left != 0 && left != APEX_TIME_INFINITY {
                let partition = Partition::find_by_id(*id).unwrap();
                let name = partition.name();
                panic!(
                    "duration left for partition '{:?}' is not zero: {:#?}",
                    name, left
                );
            }
        }

        Some((
            cpu_id,
            RuntimeSchedTable::new(time_as_duration(major_frame_size), table.into_iter()).unwrap(),
            inspectors,
        ))
    }
}

fn iter_key_value(args: &str) -> Result<impl Iterator<Item = (&str, &str)>, String> {
    let result = args.split(',').map(|a| {
        let mut a = a.split('=');
        let key = a.next().unwrap();
        let val = a.next().unwrap();
        if a.next().is_some() {
            Err(format!("invalid argument: {:?}", a))
        } else {
            Ok((key, val))
        }
    });
    for r in result.clone() {
        r?;
    }
    Ok(result.map(|r| r.unwrap()))
}

fn parse_key_value<'a: 'b, 'b>(
    mut config: impl Iterator<Item = &'b (&'a str, &'a str)>,
    key: &str,
) -> Option<&'a str> {
    config.find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn parse_usize_from_proper_redix(s: &str) -> Result<usize, ParseIntError> {
    let (radix, s) = match s {
        s if s.starts_with("0x")
            || s.starts_with("0X")
            || s.starts_with("0h")
            || s.starts_with("0H") =>
        {
            (16, &s[2..])
        }
        s if s.starts_with("0o") || s.starts_with("0O") => (8, &s[2..]),
        s if s.starts_with("0b") || s.starts_with("0B") => (2, &s[2..]),
        _ => (10, s),
    };
    usize::from_str_radix(s, radix)
}

fn parse_time_from_proper_unit(s: &str) -> Result<ApexSystemTime, &str> {
    let (unit, s) = match s {
        s if s.ends_with("ns") => (1, &s[..s.len() - 2]),
        s if s.ends_with("us") => (1000, &s[..s.len() - 2]),
        s if s.ends_with("ms") => (1000 * 1000, &s[..s.len() - 2]),
        s if s.ends_with('s') => (1000 * 1000 * 1000, &s[..s.len() - 1]),
        s if s == "0" => (1, s),
        _ => return Err("invalid time unit"),
    };

    Ok(
        match s
            .parse::<ApexSystemTime>()
            .map_err(|_| "invalid time value")?
            * unit
        {
            time if time < 0 => APEX_TIME_INFINITY,
            time => ApexSystemTime::from(time),
        },
    )
}

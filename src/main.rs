use std::{collections::HashMap, process::Command, thread, time::{Duration, SystemTime, UNIX_EPOCH}, fs::File, io::Write};
use chrono::{NaiveDateTime, TimeZone, Local};
use clap::{Arg, Command as ClapCommand};

#[derive(Debug)]
struct LogEntry {
    user: String,
    mem_mib: u64,
}

fn get_gpu_usage_snapshot() -> Vec<LogEntry> {
    let smi = Command::new("nvidia-smi")
        .args(&[
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .expect("Failed to run nvidia-smi");
    let out = String::from_utf8_lossy(&smi.stdout);

    let mut pid_mem = Vec::new();
    for line in out.lines() {
        let parts: Vec<_> = line.split(',').map(str::trim).collect();
        if parts.len() != 2 { continue; }
        if let (Ok(pid), Ok(mem)) = (parts[0].parse::<u32>(), parts[1].parse::<u64>()) {
            pid_mem.push((pid, mem));
        }
    }
    if pid_mem.is_empty() {
        return Vec::new();
    }

    let pids: Vec<String> = pid_mem.iter().map(|(pid, _)| pid.to_string()).collect();
    let ps_out = Command::new("ps")
        .args(&["-o", "pid=,user=", "-p", &pids.join(",")])
        .output()
        .expect("Failed to run ps");
    let ps_str = String::from_utf8_lossy(&ps_out.stdout);

    let mut user_map = HashMap::new();
    for line in ps_str.lines() {
        if let Some((pid_str, user)) = line.trim().split_once(' ') {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                user_map.insert(pid, user.trim().to_string());
            }
        }
    }

    pid_mem.into_iter().filter_map(|(pid, mem_mib)| {
        user_map.get(&pid).map(|user| LogEntry { user: user.clone(), mem_mib })
    }).collect()
}

fn backup_usage_snapshot(
    total_usage: &HashMap<String, f64>,
    timestamp: u64,
    prefix: &str,
    overwrite: bool,
) {
    // Determine filename
    let filename = if overwrite {
        format!("{}.csv", prefix)
    } else {
        format!("{}_{}.csv", prefix, timestamp)
    };

    // Write (or overwrite) the file
    if let Ok(mut file) = File::create(&filename) {
        writeln!(file, "user,gib_hr").ok();
        for (user, &gib_hr) in total_usage {
            writeln!(file, "{},{}", user, gib_hr).ok();
        }
        println!("[Backup] Written snapshot to {}", filename);
    } else {
        eprintln!("[Backup] Failed to create backup file {}", filename);
    }
}

fn main() {
    let matches = ClapCommand::new("gpu-monitor")
        .about("Monitor GPU memory usage per user over time, with periodic backups")
        .arg(Arg::new("end-time")
            .short('e')
            .long("end-time")
            .value_name("YYYY-MM-DD-HH:MM:SS")
            .required(true)
            .help("End datetime for monitoring (local time)"))
        .arg(Arg::new("interval")
            .short('i')
            .long("interval")
            .value_name("SECONDS")
            .required(true)
            .help("Sampling interval in seconds"))
        .arg(Arg::new("backup-interval")
            .short('b')
            .long("backup-interval")
            .value_name("SECONDS")
            .required(false)
            .default_value("300")
            .help("Interval in seconds for writing backups"))
        .arg(Arg::new("backup-mode")
            .short('m')
            .long("backup-mode")
            .value_name("MODE")
            .required(false)
            .default_value("new")
            .help("Backup mode: 'new' for timestamped files, 'overwrite' to reuse a single file"))
        .arg(Arg::new("prefix")
            .short('p')
            .long("prefix")
            .value_name("PREFIX")
            .required(false)
            .default_value("gpu_usage_backup")
            .help("Filename prefix for backups"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .help("Enable verbose snapshot output"))
        .get_matches();

    let end_str = matches.get_one::<String>("end-time").unwrap();
    let interval_sec: u64 = matches.get_one::<String>("interval").unwrap().parse().expect("Invalid interval");
    let backup_interval: u64 = matches.get_one::<String>("backup-interval").unwrap().parse().expect("Invalid backup interval");
    let mode = matches.get_one::<String>("backup-mode").unwrap().as_str();
    let overwrite = mode.eq_ignore_ascii_case("overwrite");
    let prefix = matches.get_one::<String>("prefix").unwrap();
    let verbose = matches.contains_id("verbose");

    let end_dt = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d-%H:%M:%S")
        .expect("Invalid datetime format");
    let end_ts = Local.from_local_datetime(&end_dt).single()
        .expect("Invalid or ambiguous local time").timestamp() as u64;

    let start_ts = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Time error").as_secs();
    if start_ts >= end_ts {
        eprintln!("End time must be in the future");
        std::process::exit(1);
    }

    let mut total_usage: HashMap<String, f64> = HashMap::new();
    let mut current = start_ts;
    let mut last_backup = start_ts;

    while current <= end_ts {
        let entries = get_gpu_usage_snapshot();
        if verbose {
            println!("[Snapshot @ {}] {} processes: {:?}", current, entries.len(), entries);
        }
        for entry in entries {
            let gib_hr = (entry.mem_mib as f64 * interval_sec as f64) / (1024.0 * 3600.0);
            *total_usage.entry(entry.user).or_insert(0.0) += gib_hr;
        }

        if current - last_backup >= backup_interval {
            backup_usage_snapshot(&total_usage, current, prefix, overwrite);
            last_backup = current;
        }

        thread::sleep(Duration::from_secs(interval_sec));
        current += interval_sec;
    }

    if total_usage.is_empty() {
        println!("No GPU usage detected during the monitoring period.");
        return;
    }
    println!("\n=== Total GPU usage (GiB·h) per user ===");
    for (user, gib_hr) in total_usage {
        println!("{:<15} {:>8.3}", user, gib_hr);
    }
}
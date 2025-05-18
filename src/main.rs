use std::{collections::HashMap, process::Command, thread, time::{Duration, SystemTime, UNIX_EPOCH}};
use chrono::{NaiveDateTime, TimeZone, Local};

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

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let verbose = if let Some(pos) = args.iter().position(|a| a == "--verbose") {
        args.remove(pos);
        true
    } else {
        false
    };

    if args.len() != 2 {
        eprintln!("Usage: gpu-monitor <end_datetime_YYYY-MM-DD-HH:MM:SS> <interval_sec> [--verbose]");
        std::process::exit(1);
    }
    let end_str = &args[0];
    let interval_sec: u64 = args[1].parse().expect("Invalid interval");

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

    while current <= end_ts {
        let entries = get_gpu_usage_snapshot();
        if verbose {
            println!("[Snapshot @ {}] {} processes: {:?}", current, entries.len(), entries);
        }
        for entry in entries {
            let gib_hr = (entry.mem_mib as f64 * interval_sec as f64) / (1024.0 * 3600.0);
            *total_usage.entry(entry.user).or_insert(0.0) += gib_hr;
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

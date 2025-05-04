use std::fs::{read_dir, read_link, read_to_string};
use std::io;
use std::mem::size_of;
use std::os::raw::c_void;
use std::os::unix::io::RawFd;
use std::ptr;
use std::path::PathBuf;

use libc::{c_uint, syscall, SYS_io_uring_setup};
use nix::sys::utsname::uname;

/// Structure representing io_uring parameters
#[repr(C)]
#[derive(Debug, Default)]
struct IoUringParams {
    sq_entries: u32,
    cq_entries: u32,
    flags: u32,
    sq_thread_cpu: u32,
    sq_thread_idle: u32,
    features: u32,
    wq_fd: u32,
    resv: [u32; 3],
    sq_off: IoSqringOffsets,
    cq_off: IoCqringOffsets,
}

/// Structure representing submission queue offsets
#[repr(C)]
#[derive(Debug, Default)]
struct IoSqringOffsets {
    head: u32,
    tail: u32,
    ring_mask: u32,
    ring_entries: u32,
    flags: u32,
    dropped: u32,
    array: u32,
    resv1: u32,
    resv2: u64,
}

/// Structure representing completion queue offsets
#[repr(C)]
#[derive(Debug, Default)]
struct IoCqringOffsets {
    head: u32,
    tail: u32,
    ring_mask: u32,
    ring_entries: u32,
    overflow: u32,
    cqes: u32,
    flags: u32,
    resv1: u32,
    resv2: u64,
}

/// Feature flags for io_uring
const IO_URING_FEATURES: &[(u32, &str)] = &[
    (1 << 0, "IORING_FEAT_SINGLE_MMAP"),
    (1 << 1, "IORING_FEAT_NODROP"),
    (1 << 2, "IORING_FEAT_SUBMIT_STABLE"),
    (1 << 3, "IORING_FEAT_RW_CUR_POS"),
    (1 << 4, "IORING_FEAT_CUR_PERSONALITY"),
    (1 << 5, "IORING_FEAT_FAST_POLL"),
    (1 << 6, "IORING_FEAT_POLL_32BITS"),
    (1 << 7, "IORING_FEAT_SQPOLL_NONFIXED"),
    (1 << 8, "IORING_FEAT_ENTER_EXT_ARG"),
    (1 << 9, "IORING_FEAT_REG_RW"),
    (1 << 10, "IORING_FEAT_SAFE_LINK"),
    (1 << 11, "IORING_FEAT_FAST_POLL_FULL"),
    (1 << 12, "IORING_FEAT_CQE_SKIP"),
];

/// Attempts to detect if io_uring is supported on the system
/// Returns Some(IoUringParams) if supported, None otherwise
fn detect_io_uring_support() -> io::Result<Option<IoUringParams>> {
    let mut params: IoUringParams = Default::default();
    let entries: c_uint = 1;

    let ret = unsafe {
        syscall(
            SYS_io_uring_setup,
            entries,
            &mut params as *mut IoUringParams,
        )
    };

    if ret >= 0 {
        unsafe {
            libc::close(ret as RawFd);
        }
        Ok(Some(params))
    } else {
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ENOSYS) {
            Ok(None) // System call not implemented
        } else {
            Err(err) // Other error occurred
        }
    }
}

/// Prints the available io_uring features
fn print_io_uring_features(params: &IoUringParams) {
    println!("\nReported io_uring feature flags:");
    let features = params.features;
    let mut found_features = false;

    for (mask, name) in IO_URING_FEATURES {
        if (features & mask) != 0 {
            println!("  - {}", name);
            found_features = true;
        }
    }

    if !found_features {
        println!("  (no features reported)");
    }
}

/// Gets the process name for a given PID
fn get_process_name(pid: u32) -> Option<String> {
    let path = format!("/proc/{}/comm", pid);
    read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Gets detailed process information including command line arguments and memory status
fn get_process_info(pid: u32) -> ProcessInfo {
    let mut info = ProcessInfo {
        name: get_process_name(pid).unwrap_or_else(|| "<unknown>".to_string()),
        exe_path: None,
        cmdline: None,
        memory_status: None,
        is_in_memory: false,
    };

    // Get executable path
    if let Ok(path) = read_link(format!("/proc/{}/exe", pid)) {
        info.exe_path = Some(path);
    }

    // Get command line arguments
    if let Ok(cmdline) = read_to_string(format!("/proc/{}/cmdline", pid)) {
        let args: Vec<String> = cmdline
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        if !args.is_empty() {
            info.cmdline = Some(args);
        }
    }

    // Check if process is in memory
    if let Ok(maps) = read_to_string(format!("/proc/{}/maps", pid)) {
        // Check for memory-mapped files
        let has_memory_mapped_files = maps.lines().any(|line| {
            line.contains("memfd:") || 
            line.contains("anon_inode:") ||
            line.contains("(deleted)")
        });
        info.is_in_memory = has_memory_mapped_files;

        // Get memory status
        if let Ok(status) = read_to_string(format!("/proc/{}/status", pid)) {
            let mut memory_info = MemoryInfo::default();
            
            for line in status.lines() {
                if line.starts_with("VmSize:") {
                    if let Some(size) = line.split_whitespace().nth(1) {
                        memory_info.virtual_memory = size.parse().ok();
                    }
                } else if line.starts_with("VmRSS:") {
                    if let Some(size) = line.split_whitespace().nth(1) {
                        memory_info.resident_memory = size.parse().ok();
                    }
                }
            }
            info.memory_status = Some(memory_info);
        }
    }

    info
}

/// Structure to hold process information
#[derive(Debug, Default)]
struct ProcessInfo {
    name: String,
    exe_path: Option<PathBuf>,
    cmdline: Option<Vec<String>>,
    memory_status: Option<MemoryInfo>,
    is_in_memory: bool,
}

/// Structure to hold memory information
#[derive(Debug, Default)]
struct MemoryInfo {
    virtual_memory: Option<u64>,
    resident_memory: Option<u64>,
}

/// Checks if any running processes are using io_uring
fn check_io_uring_usage() -> io::Result<()> {
    println!("\nChecking if any process is using io_uring...");

    let mut found = false;
    let proc_entries = read_dir("/proc")?;

    for entry in proc_entries.flatten() {
        if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
            let fd_dir = format!("/proc/{}/fd", pid);
            if let Ok(fds) = read_dir(fd_dir) {
                for fd_entry in fds.flatten() {
                    if let Ok(link_target) = read_link(fd_entry.path()) {
                        if link_target.to_string_lossy().contains("anon_inode:[io_uring]") {
                            let info = get_process_info(pid);
                            
                            println!("\nProcess using io_uring:");
                            println!("  PID: {}", pid);
                            println!("  Name: {}", info.name);
                            
                            if let Some(path) = info.exe_path {
                                println!("  Executable: {}", path.display());
                            } else {
                                println!("  Executable: <unavailable>");
                            }

                            if let Some(cmdline) = info.cmdline {
                                println!("  Command line: {}", cmdline.join(" "));
                            }

                            if info.is_in_memory {
                                println!("  Status: Running in memory");
                            }

                            if let Some(memory) = info.memory_status {
                                if let Some(vm) = memory.virtual_memory {
                                    println!("  Virtual Memory: {} kB", vm);
                                }
                                if let Some(rss) = memory.resident_memory {
                                    println!("  Resident Memory: {} kB", rss);
                                }
                            }

                            println!("  io_uring FD: {:?}", fd_entry.file_name());
                            found = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    if !found {
        println!("No active process is currently using io_uring.");
    }

    Ok(())
}

fn main() -> io::Result<()> {
    println!("\nChecking system information...");
    let sys = uname().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    println!("  Architecture: {}", sys.machine().to_string_lossy());
    println!("  Kernel Version: {} {}", sys.sysname().to_string_lossy(), sys.release().to_string_lossy());
    println!("  Node Name: {}", sys.nodename().to_string_lossy());
    println!("  Version: {}", sys.version().to_string_lossy());

    // Check for io_uring support
    match detect_io_uring_support()? {
        Some(params) => {
            println!("\nio_uring is supported on this system!");
            print_io_uring_features(&params);
        }
        None => {
            println!("\nio_uring is not supported on this system.");
            println!("The kernel may be too old (requires 5.1 or later) or io_uring support may be disabled.");
            return Ok(());
        }
    }

    println!("\nChecking for processes using io_uring...");
    check_io_uring_usage()?;

    Ok(())
} 
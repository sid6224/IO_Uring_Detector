use std::fs::File;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;
use io_uring::{opcode, IoUring, squeue, types};

// On-disk process example
fn on_disk_process() -> io::Result<()> {
    println!("[On-disk Process] Starting...");
    
    // Create a temporary file
    let mut file = File::create("test_file.txt")?;
    file.write_all(b"Testing io_uring with on-disk process")?;
    println!("[On-disk Process] Created test_file.txt");
    
    // Use io_uring to read the file
    let fd = types::Fd(file.as_raw_fd());
    let mut ring = IoUring::new(32)?;
    println!("[On-disk Process] Created io_uring ring");
    
    // Submit a read operation
    let mut buf = vec![0u8; 1024];
    let read_e = opcode::Read::new(fd, buf.as_mut_ptr(), buf.len() as u32)
        .build()
        .flags(squeue::Flags::empty());
    
    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("[On-disk Process] Failed to push read operation");
    }
    println!("[On-disk Process] Prepared read operation");
    
    // Submit and wait for completion
    ring.submit_and_wait(1)?;
    println!("[On-disk Process] Read operation completed");
    
    // Keep the file open and ring active for a while
    println!("[On-disk Process] Keeping file and ring active for 120 seconds...");
    thread::sleep(Duration::from_secs(120));
    
    // Clean up
    drop(file);
    std::fs::remove_file("test_file.txt")?;
    println!("[On-disk Process] Cleaned up test file");
    
    Ok(())
}

// In-memory process example
fn in_memory_process() -> io::Result<()> {
    println!("[In-memory Process] Starting...");
    
    // Create a memory buffer
    let buf = vec![0u8; 1024];
    println!("[In-memory Process] Created memory buffer");
    
    // Use io_uring to perform in-memory operations
    let mut ring = IoUring::new(32)?;
    println!("[In-memory Process] Created io_uring ring");
    
    // Submit a write operation to memory
    let write_e = opcode::Write::new(types::Fd(0), buf.as_ptr(), buf.len() as u32)
        .build()
        .flags(squeue::Flags::empty());
    
    unsafe {
        ring.submission()
            .push(&write_e)
            .expect("[In-memory Process] Failed to push write operation");
    }
    println!("[In-memory Process] Prepared write operation");
    
    // Submit and wait for completion
    ring.submit_and_wait(1)?;
    println!("[In-memory Process] Write operation completed");
    
    // Keep the ring active for a while
    println!("[In-memory Process] Keeping ring active for 120 seconds...");
    thread::sleep(Duration::from_secs(120));
    
    Ok(())
}

fn main() -> io::Result<()> {
    println!("Starting io_uring test processes...");
    println!("This program will run for 150 seconds to allow detection");
    println!("Run the detector in another terminal to see the processes");
    
    // Start on-disk process in a separate thread
    let _on_disk_handle = thread::spawn(|| {
        if let Err(e) = on_disk_process() {
            eprintln!("[On-disk Process] Error: {}", e);
        }
    });
    
    // Add a small delay before starting the in-memory process
    thread::sleep(Duration::from_secs(5));
    
    // Start in-memory process in a separate thread
    let _in_memory_handle = thread::spawn(|| {
        if let Err(e) = in_memory_process() {
            eprintln!("[In-memory Process] Error: {}", e);
        }
    });
    
    // Keep the program running for 150 seconds
    println!("Processes will run for 150 seconds...");
    thread::sleep(Duration::from_secs(150));
    
    // The threads will automatically clean up when the program exits
    println!("Test completed. You can now stop the detector.");
    Ok(())
} 
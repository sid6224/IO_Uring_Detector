use std::fs::File;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

// On-disk process example
fn on_disk_process() -> io::Result<()> {
    println!("[On-disk Process] Starting...");
    
    // Create a temporary file
    let mut file = File::create("test_file.txt")?;
    file.write_all(b"Testing io_uring with on-disk process")?;
    println!("[On-disk Process] Created test_file.txt");
    
    // Use io_uring to read the file
    let fd = file.as_raw_fd();
    let mut ring = io_uring::IoUring::new(32)?;
    println!("[On-disk Process] Created io_uring ring");
    
    // Submit a read operation
    let mut buf = vec![0u8; 1024];
    let sqe = ring.submission().next().unwrap();
    unsafe {
        sqe.prepare_read(fd, &mut buf, 0);
    }
    println!("[On-disk Process] Prepared read operation");
    
    // Submit and wait for completion
    ring.submit()?;
    ring.submit_and_wait(1)?;
    println!("[On-disk Process] Read operation completed");
    
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
    let mut buf = vec![0u8; 1024];
    println!("[In-memory Process] Created memory buffer");
    
    // Use io_uring to perform in-memory operations
    let mut ring = io_uring::IoUring::new(32)?;
    println!("[In-memory Process] Created io_uring ring");
    
    // Submit a write operation to memory
    let sqe = ring.submission().next().unwrap();
    unsafe {
        sqe.prepare_write(0, &buf, 0); // Using stdin as a placeholder
    }
    println!("[In-memory Process] Prepared write operation");
    
    // Submit and wait for completion
    ring.submit()?;
    ring.submit_and_wait(1)?;
    println!("[In-memory Process] Write operation completed");
    
    Ok(())
}

fn main() -> io::Result<()> {
    println!("Starting io_uring test processes...");
    println!("This program will run for 30 seconds to allow detection");
    println!("Run the detector in another terminal to see the processes");
    
    // Start on-disk process in a separate thread
    let on_disk_handle = thread::spawn(|| {
        if let Err(e) = on_disk_process() {
            eprintln!("[On-disk Process] Error: {}", e);
        }
    });
    
    // Start in-memory process in a separate thread
    let in_memory_handle = thread::spawn(|| {
        if let Err(e) = in_memory_process() {
            eprintln!("[In-memory Process] Error: {}", e);
        }
    });
    
    // Keep the program running for 30 seconds
    println!("Processes will run for 30 seconds...");
    thread::sleep(Duration::from_secs(30));
    
    // The threads will automatically clean up when the program exits
    println!("Test completed. You can now stop the detector.");
    Ok(())
} 
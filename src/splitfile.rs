/// Split / Combine files (inspired by DN's FBB.PAS)
/// Split: breaks a file into sized chunks (.001, .002, etc.)
/// Combine: joins chunk files back into the original

use std::fs::{self, File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Default chunk size: 1 MB
pub const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024;

/// Split a file into chunks of given size
pub fn split_file(path: &Path, chunk_size: u64, dest_dir: &Path) -> Result<usize, String> {
    let file = File::open(path)
        .map_err(|e| format!("Cannot open file: {}", e))?;
    let file_size = file.metadata()
        .map_err(|e| format!("Cannot read metadata: {}", e))?
        .len();

    if file_size == 0 {
        return Err("File is empty".to_string());
    }

    let stem = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let mut reader = BufReader::new(file);
    let mut chunk_num = 0usize;
    let mut remaining = file_size;
    let mut buf = vec![0u8; 65536.min(chunk_size as usize)]; // 64K read buffer

    while remaining > 0 {
        chunk_num += 1;
        let chunk_name = format!("{}.{:03}", stem, chunk_num);
        let chunk_path = dest_dir.join(&chunk_name);

        let mut out = BufWriter::new(
            File::create(&chunk_path)
                .map_err(|e| format!("Cannot create {}: {}", chunk_name, e))?
        );

        let mut written = 0u64;
        let to_write = chunk_size.min(remaining);

        while written < to_write {
            let read_size = buf.len().min((to_write - written) as usize);
            let n = reader.read(&mut buf[..read_size])
                .map_err(|e| format!("Read error: {}", e))?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .map_err(|e| format!("Write error: {}", e))?;
            written += n as u64;
        }

        out.flush().map_err(|e| format!("Flush error: {}", e))?;
        remaining = remaining.saturating_sub(to_write);
    }

    Ok(chunk_num)
}

/// Combine chunk files back into original
/// Detects chunks by looking for .001, .002, .003, etc.
pub fn combine_files(first_chunk: &Path, dest: &Path) -> Result<u64, String> {
    let stem = first_chunk.file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid chunk filename")?;
    let parent = first_chunk.parent().unwrap_or(Path::new("."));

    let mut out = BufWriter::new(
        File::create(dest)
            .map_err(|e| format!("Cannot create output: {}", e))?
    );

    let mut total = 0u64;
    let mut chunk_num = 1usize;
    let mut buf = vec![0u8; 65536];

    loop {
        let chunk_name = format!("{}.{:03}", stem, chunk_num);
        let chunk_path = parent.join(&chunk_name);

        if !chunk_path.exists() {
            break;
        }

        let file = File::open(&chunk_path)
            .map_err(|e| format!("Cannot open {}: {}", chunk_name, e))?;
        let mut reader = BufReader::new(file);

        loop {
            let n = reader.read(&mut buf)
                .map_err(|e| format!("Read error: {}", e))?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .map_err(|e| format!("Write error: {}", e))?;
            total += n as u64;
        }

        chunk_num += 1;
    }

    out.flush().map_err(|e| format!("Flush error: {}", e))?;

    if chunk_num == 1 {
        // No chunks found
        let _ = fs::remove_file(dest);
        return Err("No chunk files found (.001, .002, ...)".to_string());
    }

    Ok(total)
}

/// Check if a file looks like a split chunk (has .001 extension)
pub fn is_first_chunk(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "001")
        .unwrap_or(false)
}

/// Detect chunk files and return count
pub fn count_chunks(first_chunk: &Path) -> usize {
    let stem = match first_chunk.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return 0,
    };
    let parent = first_chunk.parent().unwrap_or(Path::new("."));
    let mut count = 0;
    loop {
        let chunk_name = format!("{}.{:03}", stem, count + 1);
        if !parent.join(&chunk_name).exists() {
            break;
        }
        count += 1;
    }
    count
}

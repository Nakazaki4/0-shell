use chrono::{DateTime, Duration, Local};
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::Path;
use users::{get_group_by_gid, get_user_by_uid};

// The new enum to handle two types of data in the size column.
enum SizeOrDevice {
    Size(u64),
    Device { major: u64, minor: u64 },
}

struct FileInfo {
    permissions: String,
    nlinks: u64,
    user_name: String,
    group_name: String,
    size_or_device: SizeOrDevice,
    mod_time: String,
    display_name: String,
}

pub fn list(args: &Vec<String>) {
    let mut show_hidden = false;
    let mut classify = false;
    let mut l_flag = false;
    let mut paths = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            for c in arg.chars().skip(1) {
                match c {
                    'a' => show_hidden = true,
                    'l' => l_flag = true,
                    'F' => classify = true,
                    _ => {
                        eprintln!("ls: invalid option -- '{}'", c);
                        return;
                    }
                }
            }
        } else {
            paths.push(arg.to_string());
        }
    }

    if paths.is_empty() {
        paths.push(".".to_string());
    }

    let multiple_paths = paths.len() > 1;

    for (i, path) in paths.iter().enumerate() {
        if multiple_paths {
            println!("{}:", path);
        }

        list_directory(path, show_hidden, l_flag, classify);

        if multiple_paths && i < paths.len() - 1 {
            println!();
        }
    }
}

// logic for listing a single directory.
fn list_directory(path_str: &str, show_hidden: bool, l_flag: bool, classify: bool) {
    let base_path = Path::new(path_str);
    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("ls: cannot access '{}': {}", path_str, e);
            return;
        }
    };

    let mut files = vec![];
    for res in entries {
        if let Ok(entry) = res {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if show_hidden || !file_name.starts_with('.') {
                files.push(file_name);
            }
        }
    }

    if show_hidden {
        files.push(".".to_string());
        files.push("..".to_string());
    }

    sort(&mut files);

    if !l_flag {
        // Simple case: if not -l, just print names and exit.
        for file_name in files {
            let mut ress = sanitize_filename(&file_name);
            if classify {
                let full_path = base_path.join(&file_name);
                if let Ok(metadata) = fs::symlink_metadata(&full_path) {
                    ress.push_str(&get_classifier(&metadata));
                }
            }
            print!("{}  ", ress);
        }
        println!();
        return;
    }

    // --- Start of -l logic ---
    let mut file_infos = Vec::new();
    let mut total_blocks = 0;

    // --- GATHER INFO AND CALCULATE TOTAL BLOCKS ---
    for file_name in &files {
        let full_path = base_path.join(file_name);
        if let Ok(metadata) = fs::symlink_metadata(&full_path) {
            total_blocks += metadata.blocks();

            let file_type = metadata.file_type();
            let size_or_device = if file_type.is_char_device() || file_type.is_block_device() {
                let rdev = metadata.rdev();
                SizeOrDevice::Device {
                    major: major(rdev),
                    minor: minor(rdev),
                }
            } else {
                SizeOrDevice::Size(metadata.len())
            };

            let mut display_name = sanitize_filename(&file_name);
            let is_symlink = metadata.file_type().is_symlink();
            if classify && !is_symlink {
                display_name.push_str(&get_classifier(&metadata));
            }
            if is_symlink {
                if let Ok(target) = fs::read_link(&full_path) {
                    let mut target_display = target.to_string_lossy().to_string();

                    if classify {
                        let full_target_path = base_path.join(&target);
                        if let Ok(target_meta) = fs::metadata(&full_target_path) {
                            target_display.push_str(&get_classifier(&target_meta));
                        }
                    }
                    display_name = format!("{} -> {}", display_name, target_display.to_string());
                }
            }

            file_infos.push(FileInfo {
                permissions: get_permissions(&metadata, &full_path),
                nlinks: metadata.nlink(),
                user_name: get_user_by_uid(metadata.uid()).map_or_else(
                    || metadata.uid().to_string(),
                    |u| u.name().to_string_lossy().into_owned(),
                ),
                group_name: get_group_by_gid(metadata.gid()).map_or_else(
                    || metadata.gid().to_string(),
                    |g| g.name().to_string_lossy().into_owned(),
                ),
                size_or_device,
                mod_time: format_time(&metadata),
                display_name,
            });
        }
    }

    // --- CALCULATE MAX WIDTHS FOR ALIGNMENT (MODIFIED SECTION) ---
    let max_links_width = file_infos
        .iter()
        .map(|fi| fi.nlinks.to_string().len())
        .max()
        .unwrap_or(0);
    let max_user_width = file_infos
        .iter()
        .map(|fi| fi.user_name.len())
        .max()
        .unwrap_or(0);
    let max_group_width = file_infos
        .iter()
        .map(|fi| fi.group_name.len())
        .max()
        .unwrap_or(0);

    let mut max_size_width = 0;
    let mut max_major_width = 0;
    let mut max_minor_width = 0;
    let mut has_device_file = false;

    for fi in &file_infos {
        match fi.size_or_device {
            SizeOrDevice::Size(size) => {
                let size_len = size.to_string().len();
                if size_len > max_size_width {
                    max_size_width = size_len;
                }
            }
            SizeOrDevice::Device { major, minor } => {
                has_device_file = true;
                let major_len = major.to_string().len();
                let minor_len = minor.to_string().len();
                if major_len > max_major_width {
                    max_major_width = major_len;
                }
                if minor_len > max_minor_width {
                    max_minor_width = minor_len;
                }
            }
        }
    }

    let total_device_space = if has_device_file {
        max_major_width + 2 + max_minor_width // Width is major + ", " + minor
    } else {
        0
    };

    let final_width = std::cmp::max(max_size_width, total_device_space);

    // --- PRINT FORMATTED OUTPUT (MODIFIED SECTION) ---
    println!("total {}", total_blocks / 2);
    for fi in file_infos {
        let size_or_device_str = match fi.size_or_device {
            SizeOrDevice::Size(size) => {
                format!("{:>width$}", size, width = final_width)
            }
            SizeOrDevice::Device { major, minor } => {
                let formatted_device = format!(
                    "{:>major_w$}, {:>minor_w$}",
                    major,
                    minor,
                    major_w = max_major_width,
                    minor_w = max_minor_width
                );
                format!("{:>width$}", formatted_device, width = final_width)
            }
        };

        println!(
            "{} {:>links_w$} {:<user_w$} {:<group_w$} {} {} {}",
            fi.permissions,
            fi.nlinks,
            fi.user_name,
            fi.group_name,
            size_or_device_str,
            fi.mod_time,
            fi.display_name,
            links_w = max_links_width,
            user_w = max_user_width,
            group_w = max_group_width
        );
    }
}

fn get_classifier(metadata: &fs::Metadata) -> String {
    let file_type = metadata.file_type();
    if file_type.is_dir() {
        "/".to_string()
    } else if file_type.is_symlink() {
        "@".to_string()
    } else if metadata.permissions().mode() & 0o111 != 0 {
        "*".to_string()
    } else if file_type.is_fifo() {
        "|".to_string()
    } else if file_type.is_socket() {
        "=".to_string()
    } else {
        "".to_string()
    }
}

fn get_permissions(metadata: &fs::Metadata, path: &Path) -> String {
    let mode = metadata.mode();
    let file_type = metadata.file_type();

    // Determine the file type character
    let file_char = if file_type.is_dir() {
        'd'
    } else if file_type.is_symlink() {
        'l'
    } else if file_type.is_fifo() {
        'p'
    } else if file_type.is_block_device() {
        'b'
    } else if file_type.is_char_device() {
        'c'
    } else if file_type.is_socket() {
        's'
    } else {
        '-'
    };

    // Permission characters
    let mut perms = String::new();
    perms.push(file_char);

    // Special mode bits
    let setuid = mode & 0o4000 != 0;
    let setgid = mode & 0o2000 != 0;
    let sticky = mode & 0o1000 != 0;

    for i in 0..3 {
        let shift = (2 - i) * 3;
        let bits = (mode >> shift) & 0b111;
        let r = if bits & 0o4 != 0 { 'r' } else { '-' };
        let w = if bits & 0o2 != 0 { 'w' } else { '-' };
        let mut x = if bits & 0o1 != 0 { 'x' } else { '-' };

        if i == 0 && setuid {
            x = if x == 'x' { 's' } else { 'S' };
        } else if i == 1 && setgid {
            x = if x == 'x' { 's' } else { 'S' };
        } else if i == 2 && sticky {
            x = if x == 'x' { 't' } else { 'T' };
        }

        perms.push(r);
        perms.push(w);
        perms.push(x);
    }

    if let Ok(path_cstring) = CString::new(path.as_os_str().as_bytes()) {
        unsafe {
            let result = libc::listxattr(path_cstring.as_ptr(), std::ptr::null_mut(), 0);
            if result > 0 {
                perms.push('+');
            }
        }
    }

    perms
}

fn format_time(metadata: &fs::Metadata) -> String {
    let time = metadata.modified().unwrap();
    let mut datetime: DateTime<Local> = time.into();

    let six_months_ago = Local::now() - Duration::days(182);
    datetime = datetime + Duration::hours(1);
    if datetime < six_months_ago {
        datetime.format("%b %e  %Y").to_string()
    } else {
        datetime.format("%b %e %H:%M").to_string()
    }
}

fn sort(files: &mut Vec<String>) {
    files.sort_by(|a: &String, b| {
        let a_normal = a.trim_start_matches('.').to_lowercase();
        let b_normal = b.trim_start_matches('.').to_lowercase();
        a_normal.cmp(&b_normal)
    });
}

fn sanitize_filename(name: &str) -> String {
    let mut needs_simple_quoting = false;
    let mut has_control_chars = false;

    // First, scan the string to decide which quoting style is needed.
    for c in name.chars() {
        if c.is_control() {
            has_control_chars = true;
            break;
        }
        if " '\\$*?&|;()<>".contains(c) {
            needs_simple_quoting = true;
        }
    }

    // --- Case 1: Complex ANSI-C quoting for tabs, newlines, etc. ---
    if has_control_chars {
        let mut result = String::new();
        let mut buf = String::new();
        for c in name.chars() {
            match c {
                '\t' => {
                    if !buf.is_empty() {
                        result.push_str(&format!("'{}'", buf));
                        buf.clear();
                    }
                    result.push_str("$'\\t'");
                }
                '\n' => {
                    if !buf.is_empty() {
                        result.push_str(&format!("'{}'", buf));
                        buf.clear();
                    }
                    result.push_str("$'\\n'");
                }
                '\'' => {
                    if !buf.is_empty() {
                        result.push_str(&format!("'{}'", buf));
                        buf.clear();
                    }
                    result.push_str("'\\''");
                }
                '\\' => {
                    if !buf.is_empty() {
                        result.push_str(&format!("'{}'", buf));
                        buf.clear();
                    }
                    result.push_str("$'\\\\'");
                }
                _ if c.is_control() => {
                    if !buf.is_empty() {
                        result.push_str(&format!("'{}'", buf));
                        buf.clear();
                    }
                    result.push_str(&format!("$'\\x{:02x}'", c as u32));
                }
                _ => buf.push(c),
            }
        }
        if !buf.is_empty() {
            result.push_str(&format!("'{}'", buf));
        }
        return result;
    }

    // --- Case 2: Simple quoting for spaces, backslashes, etc. ---
    if needs_simple_quoting {
        return format!("'{}'", name.replace('\'', "'\\''"));
    }

    // --- Case 3: No special characters ---
    name.to_string()
}

fn major(dev: u64) -> u64 {
    (dev >> 8) & 0xfff
}

fn minor(dev: u64) -> u64 {
    (dev & 0xff) | ((dev >> 12) & 0xfff00)
}
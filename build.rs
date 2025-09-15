use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Chrome 다운로드 스킵 - 의존성만 설치
}

fn download_chrome(chrome_dir: &Path) {
    use reqwest::blocking;
    use std::io::Write;
    
    let chrome_url = if cfg!(target_os = "windows") {
        "https://storage.googleapis.com/chromium-browser-snapshots/Win_x64/1097615/chrome-win.zip"
    } else if cfg!(target_os = "macos") {
        "https://storage.googleapis.com/chromium-browser-snapshots/Mac/1097615/chrome-mac.zip"
    } else {
        "https://storage.googleapis.com/chromium-browser-snapshots/Linux_x64/1097615/chrome-linux.zip"
    };
    
    println!("Downloading Chrome from: {}", chrome_url);
    
    let response = blocking::get(chrome_url)
        .expect("Failed to download Chrome");
    
    let zip_path = chrome_dir.join("chrome.zip");
    let mut file = fs::File::create(&zip_path)
        .expect("Failed to create zip file");
    
    file.write_all(&response.bytes().expect("Failed to read response"))
        .expect("Failed to write zip file");
    
    extract_zip(&zip_path, chrome_dir);
    fs::remove_file(&zip_path).ok();
    
    println!("Chrome downloaded and extracted to: {}", chrome_dir.display());
}

fn extract_zip(zip_path: &Path, extract_to: &Path) {
    use zip::ZipArchive;
    use std::fs::File;
    use std::io::{Read, Write};
    
    let file = File::open(zip_path).expect("Failed to open zip file");
    let mut archive = ZipArchive::new(file).expect("Failed to read zip archive");
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Failed to read zip entry");
        let outpath = extract_to.join(file.name());
        
        if file.is_dir() {
            fs::create_dir_all(&outpath).expect("Failed to create directory");
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).expect("Failed to create parent directory");
                }
            }
            let mut outfile = File::create(&outpath).expect("Failed to create file");
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("Failed to read file");
            outfile.write_all(&buffer).expect("Failed to write file");
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if file.name().ends_with(".exe") || file.name().contains("chrome") {
                    let mut perms = outfile.metadata().unwrap().permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&outpath, perms).ok();
                }
            }
        }
    }
}
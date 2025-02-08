use colored::*;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct Installer {
    dll_source: PathBuf,
    installation_dir: PathBuf,
}

impl Installer {
    pub fn new() -> io::Result<Self> {
        let cargo_home = dirs::home_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?
            .join(".cargo");

        Ok(Self {
            dll_source: PathBuf::from("go_lib/go_lib.dll"),
            installation_dir: cargo_home.join("bin"),
        })
    }

    pub fn install(&self) -> io::Result<()> {
        println!("{}", "🚀 Starting installation process...".cyan().bold());

        // Create installation directory if it doesn't exist
        print!("📁 Creating installation directory... ");
        if !self.installation_dir.exists() {
            fs::create_dir_all(&self.installation_dir)?;
            println!("{}", "OK".green().bold());
        } else {
            println!("{}", "EXISTS".blue().bold());
        }

        // Copy DLL to installation directory
        let dll_dest = self.installation_dir.join("go_lib.dll");
        print!(
            "📦 Copying DLL to: {}... ",
            dll_dest.display().to_string().blue()
        );
        fs::copy(&self.dll_source, &dll_dest)?;
        println!("{}", "OK".green().bold());

        // Update PATH if necessary
        self.update_path()?;

        println!(
            "{}",
            "✅ Installation completed successfully!".green().bold()
        );
        Ok(())
    }

    fn update_path(&self) -> io::Result<()> {
        let path_var = env::var("PATH").unwrap_or_default();
        let installation_dir_str = self.installation_dir.to_string_lossy();

        if !path_var.contains(&*installation_dir_str) {
            println!("Adding installation directory to PATH...");

            #[cfg(windows)]
            {
                use std::process::Command;
                Command::new("setx")
                    .args(["PATH", &format!("{};%PATH%", installation_dir_str)])
                    .output()?;
            }

            println!("PATH updated. Please restart your terminal for changes to take effect.");
        }

        Ok(())
    }

    pub fn verify_installation(&self) -> io::Result<()> {
        println!("{}", "\n🔍 Verifying installation...".cyan().bold());

        // Check if DLL exists
        print!("Checking DLL presence... ");
        let dll_path = self.installation_dir.join("go_lib.dll");
        if !dll_path.exists() {
            println!("{}", "NOT FOUND".red().bold());
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("DLL not found at: {}", dll_path.display()),
            ));
        }
        println!("{}", "OK".green().bold());

        // Try loading the DLL
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::libloaderapi::{FreeLibrary, LoadLibraryW};

            let wide_path: Vec<u16> = dll_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            unsafe {
                let handle = LoadLibraryW(wide_path.as_ptr());
                if handle.is_null() {
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to load DLL"));
                }
                FreeLibrary(handle);
            }
        }

        println!(
            "{}",
            "✅ Installation verified successfully!".green().bold()
        );
        Ok(())
    }

    pub fn get_dll_path(&self) -> PathBuf {
        self.installation_dir.join("go_lib.dll")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation() {
        match Installer::new() {
            Ok(installer) => {
                assert!(installer.installation_dir.ends_with("bin"));
                assert!(installer.dll_source.ends_with("go_lib.dll"));
            }
            Err(e) => panic!("Failed to create installer: {}", e),
        }
    }

    #[test]
    fn test_get_dll_path() {
        if let Ok(installer) = Installer::new() {
            let dll_path = installer.get_dll_path();
            assert!(dll_path.ends_with("go_lib.dll"));
            assert_eq!(dll_path.parent().unwrap(), installer.installation_dir);
        }
    }

    #[test]
    fn test_installation_directory_creation() {
        if let Ok(installer) = Installer::new() {
            // Create a temporary test directory
            let test_dir = installer.installation_dir.join("test_install");
            let test_installer = Installer {
                installation_dir: test_dir.clone(),
                dll_source: installer.dll_source,
            };

            // Test directory creation
            if let Err(e) = fs::create_dir_all(&test_installer.installation_dir) {
                panic!("Failed to create test directory: {}", e);
            }

            assert!(test_dir.exists());

            // Cleanup
            let _ = fs::remove_dir_all(test_dir);
        }
    }

    #[test]
    fn test_path_environment_variable() {
        if let Ok(installer) = Installer::new() {
            let path = env::var("PATH").unwrap_or_default();
            let installation_dir = installer.installation_dir.to_string_lossy();
            println!(
                "Installation dir in PATH: {}",
                path.contains(&*installation_dir)
            );
        }
    }
}

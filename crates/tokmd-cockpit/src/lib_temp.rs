// ... imports

impl AsRef<str> for FileStat {
    fn as_ref(&self) -> &str {
        &self.path
    }
}

// ... rest of the file

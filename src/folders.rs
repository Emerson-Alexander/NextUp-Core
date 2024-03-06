use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// Represents a folder in the folders table.
///
/// # Fields
///
/// * `id` - Unique identifier for the folder.
/// * `parent_id` - Identifier of the parent folder. Root folders are None.
/// * `name` - The name of the folder.
/// * `style` - The functional style of the folder, as defined by the `Style` enum.
/// * `status` - A numerical status code representing the folder's current state or condition. Specific meanings are context-dependent.
pub struct Folder {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub name: String,
    pub style: Style,
    pub status: Option<u32>,
}

/// Enumerates the different styles a folder can have.
///
/// This affects how the folder is interacted with.
#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    /// Represents a standard directory that can contain files and other directories.
    Directory,
    /// Represents a set of tasks. Only one will be displayed and all will reset simultaniously.
    Selector,
    /// Represents a set of tasks. Only one will be displayed at a time, the next displays once it is completed. All reset when the last task is completed.
    Iterator,
}

/// Provides a human-readable representation of the folder style.
impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Style::Directory => write!(f, "Directory"),
            Style::Selector => write!(f, "Selector"),
            Style::Iterator => write!(f, "Iterator"),
        }
    }
}

/// Implements parsing from a string slice to a `Style` enum.
///
/// This allows for easy conversion from textual representations (e.g., the database) into the strongly typed `Style` enum.
impl FromStr for Style {
    type Err = ParseStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Directory" => Ok(Style::Directory),
            "Selector" => Ok(Style::Selector),
            "Iterator" => Ok(Style::Iterator),
            _ => Err(ParseStyleError::InvalidInput(s.to_string())),
        }
    }
}

/// Defines errors that can occur when parsing a string into a `Style`.
#[derive(Debug, Clone)]
pub enum ParseStyleError {
    /// Indicates that the input string does not correspond to any known `Style`.
    ///
    /// Contains the invalid input to aid in debugging or error reporting.
    InvalidInput(String),
}

/// Implements display formatting for `ParseStyleError`, providing a human-readable description of the error.
impl fmt::Display for ParseStyleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseStyleError::InvalidInput(input) => write!(f, "Invalid input: {}", input),
        }
    }
}

/// Allows `ParseStyleError` to integrate with Rust's standard error handling mechanisms.
impl Error for ParseStyleError {}

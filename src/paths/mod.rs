pub mod access_management;
pub mod adding_files;
pub mod adding_tags;
pub mod common;

pub trait Path {
    fn get_path() -> String;
}

pub mod bloom;
pub mod cuckoo;
pub mod theory;

pub trait FilterParameters {
    fn error(&self) -> Option<f64>;

    fn elements(&self) -> Option<u64>;

    fn storage(&self) -> Option<u64>;

    fn bits_per_element(&self) -> Option<f64>;
}

#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_slice)]

mod bplus_tree;
mod get;
mod insert;
mod map;
mod remove;
mod bulk_loading;

pub use bplus_tree::BPlusTreeMap;
pub use map::*;
pub use bulk_loading::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

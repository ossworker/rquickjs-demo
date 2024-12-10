mod loader;
mod module_builder;

use rquickjs::{embed, loader::Bundle};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

static BUNDLE: Bundle = embed! {
    // "myModule": "js/my_module.js",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

include!("tests/part_01.rs");

#[cfg(test)]
#[path = "tests/main_tests.rs"]
mod tests;

#[cfg(test)]
include!("tests/dialog.rs");
#[cfg(test)]
include!("tests/window.rs");
#[cfg(test)]
include!("tests/dialog_builtin_validation.rs");

#[cfg(test)]
#[path = "tests/builders.rs"]
mod builders_tests;

#[cfg(test)]
#[path = "tests/core.rs"]
mod core_tests;

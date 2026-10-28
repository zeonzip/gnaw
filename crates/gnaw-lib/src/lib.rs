pub mod arg;
pub mod os;

#[macro_export]
macro_rules! shell {
    ($($cmd:expr);+ $(;)?) => {
        $(
            ::std::process::Command::new("sh")
                .arg("-c")
                .arg($cmd)
                .status()
                .unwrap();
        )*
    };
}

pub use gnaw_lib_macros::task;
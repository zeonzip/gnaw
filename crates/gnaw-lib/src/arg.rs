use std::ffi::OsString;

pub trait ConsoleArgs {
    fn from_args<I: Iterator<Item = OsString>>(args: &mut I) -> Option<Self>
        where Self: Sized;
}

macro_rules! impl_arg_tuple {
    ($($T:ident),+) => {
        impl<$($T: ConsoleArgs),+> ConsoleArgs for ($($T,)+) {
            fn from_args<I: Iterator<Item=OsString>>(args: &mut I) -> Option<Self> {
                Some(($($T::from_args(args)?,)+))
            }
        }
    };
}

macro_rules! impl_arg_tuples {
    () => {};
    ($head:ident $(, $tail:ident)*) => {
        impl_arg_tuple!($head $(, $tail)*);
        impl_arg_tuples!($($tail),*);
    };
}

impl_arg_tuples!(A, B, C, D, E, F, G, H, Z, J, K, L);

macro_rules! for_parse {
    ($($T:ty),+) => {$(
        impl ConsoleArgs for $T {
            fn from_args<I: Iterator<Item = OsString>>(args: &mut I) -> Option<Self> {
                args.next()?.to_str()?.parse().ok()
            }
        }
    )+}
}

for_parse!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64,
    bool, char, String,
    std::net::IpAddr, std::net::SocketAddr
);

impl<T: ConsoleArgs> ConsoleArgs for Option<T> {
    fn from_args<I: Iterator<Item=OsString>>(args: &mut I) -> Option<Self>
    where
        Self: Sized,
    {
        Some(T::from_args(args))
    }
}

pub struct Clap<T>(pub T);

impl<T> Clap<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: clap::Parser> ConsoleArgs for Clap<T> {
    fn from_args<I: Iterator<Item=OsString>>(args: &mut I) -> Option<Self> {
        let mut matches = T::command()
            .no_binary_name(true)
            .try_get_matches_from(args)
            .ok()?;

        Some(Clap(T::from_arg_matches_mut(&mut matches).ok()?))
    }
}
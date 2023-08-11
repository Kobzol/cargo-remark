use std::fmt::Debug;

pub trait ForceUnwrap {
    type Target;

    fn unwrap_force(self) -> Self::Target;
}

impl<T> ForceUnwrap for Option<T> {
    type Target = T;

    fn unwrap_force(self) -> Self::Target {
        #[cfg(feature = "force-unwrap")]
        match self {
            Some(x) => x,
            None => unsafe {
                std::hint::unreachable_unchecked();
            },
        }
        #[cfg(not(feature = "force-unwrap"))]
        self.unwrap()
    }
}

impl<T, E> ForceUnwrap for Result<T, E>
where
    E: Debug,
{
    type Target = T;

    fn unwrap_force(self) -> Self::Target {
        #[cfg(feature = "force-unwrap")]
        match self {
            Ok(x) => x,
            Err(_) => unsafe {
                std::hint::unreachable_unchecked();
            },
        }
        #[cfg(not(feature = "force-unwrap"))]
        self.unwrap()
    }
}

#[macro_export]
macro_rules! measure {
    ($name: expr, $block: block) => {{
        let start = std::time::SystemTime::now();
        let ret = $block;
        eprintln!("{}: {} ms", $name, start.elapsed().unwrap().as_millis());
        ret
    }};
}

pub trait Sliced<T> {
    fn slice(&self, start: usize, end: usize) -> &[T];
}

impl<'a, T> Sliced<T> for &'a [T] {
    #[inline(always)]
    fn slice(&self, start: usize, len: usize) -> &[T] {
        unsafe {
            let start = self.as_ptr().add(start);
            std::slice::from_raw_parts(start, len)
        }
    }
}

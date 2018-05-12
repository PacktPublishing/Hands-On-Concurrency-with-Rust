use std::iter::Iterator;
use std::mem;

macro_rules! unsized_iter {
    ($name:ident, $int:ty, $max:expr) => {
        #[derive(Default)]
        pub struct $name {
            cur: $int,
            done: bool,
        }

        impl Iterator for $name {
            type Item = $int;

            fn next(&mut self) -> Option<$int> {
                if self.done {
                    return None;
                }
                let old = self.cur;
                if old == $max {
                    self.done = true;
                }
                self.cur = self.cur.saturating_add(1);
                return Some(old);
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let size = mem::size_of::<$int>() * 8;
                let total = 2_usize.pow(size as u32);
                let remaining = total - (self.cur as usize);
                (remaining, Some(remaining))
            }
        }
    };
}

unsized_iter!(SmallU8, u8, u8::max_value());
unsized_iter!(SmallU16, u16, u16::max_value());
unsized_iter!(SmallU32, u32, u32::max_value());
unsized_iter!(SmallU64, u64, u64::max_value());
unsized_iter!(SmallUsize, usize, usize::max_value());

macro_rules! sized_iter {
    ($name:ident, $int:ty, $min:expr) => {
        #[derive(Default)]
        pub struct $name {
            cur: $int,
            done: bool,
        }

        impl Iterator for $name {
            type Item = $int;

            fn next(&mut self) -> Option<$int> {
                if self.done {
                    return None;
                }

                let old = self.cur;
                if self.cur == 0 {
                    self.cur = -1;
                } else if self.cur.is_negative() && self.cur == $min {
                    self.done = true;
                } else if self.cur.is_positive() {
                    self.cur *= -1;
                    self.cur -= 1;
                } else if self.cur.is_negative() {
                    self.cur *= -1;
                }
                return Some(old);
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let size = mem::size_of::<$int>() * 8;
                let total = 2_usize.pow(size as u32);
                let remaining = total - ((self.cur.abs() * 2) as usize);
                (remaining, Some(remaining))
            }
        }
    };
}

sized_iter!(SmallI8, i8, i8::min_value());
sized_iter!(SmallI16, i16, i16::min_value());
sized_iter!(SmallI32, i32, i32::min_value());
sized_iter!(SmallI64, i64, i64::min_value());
sized_iter!(SmallIsize, isize, isize::min_value());

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! sized_iter_test {
        ($test_name:ident, $iter_name:ident, $int:ty) => {
            #[test]
            fn $test_name() {
                let i = $iter_name::default();
                let size = mem::size_of::<$int>() as u32;
                assert_eq!(i.count(), 2_usize.pow(size * 8));
            }
        };
    }

    sized_iter_test!(i8_count_correct, SmallI8, i8);
    sized_iter_test!(i16_count_correct, SmallI16, i16);
    sized_iter_test!(i32_count_correct, SmallI32, i32);
    // NOTE: These take forever. Uncomment if you have time to burn.
    // sized_iter_test!(i64_count_correct, SmallI64, i64);
    // sized_iter_test!(isize_count_correct, SmallIsize, isize);

    macro_rules! unsized_iter_test {
        ($test_name:ident, $iter_name:ident, $int:ty) => {
            #[test]
            fn $test_name() {
                let i = $iter_name::default();
                let size = mem::size_of::<$int>() as u32;
                assert_eq!(i.count(), 2_usize.pow(size * 8));
            }
        };
    }

    unsized_iter_test!(u8_count_correct, SmallU8, u8);
    unsized_iter_test!(u16_count_correct, SmallU16, u16);
    unsized_iter_test!(u32_count_correct, SmallU32, u32);
    // NOTE: These take forever. Uncomment if you have time to burn.
    // unsized_iter_test!(u64_count_correct, SmallU64, u64);
    // unsized_iter_test!(usize_count_correct, SmallUsize, usize);
}

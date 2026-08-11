#![feature(generic_const_exprs)]

use std::mem;
use num_integer::Integer;
use std::ops::Not;

/// Integer that can describe number of buckets, and can be the index of a bucket.
type RadixInt = u8;

/// Unsigned int required to describe the index of a digit in a number.
/// Like usize, but only for indexing Radixed objects.
/// `u8` is enough only as long as those objects have less than 256 digits.
type DigitIndexInt = u8;

// trait Radixed<const RADIX: RadixInt> {
//
//     type Digit: Ord + Into<RadixInt>;
//
//     fn digit(&self, i: DigitIndexInt) -> Self::Digit;
//
//     fn num_digits(&self) -> DigitIndexInt;
//
//     fn sort(mut vec: &mut [Self]) where Self: Sized{
//         todo!()
//     }
// }

trait RadixedCopy<const RADIX: RadixInt>: Copy {

    type Digit: Ord + Into<RadixInt>;

    fn digit(self, i: DigitIndexInt) -> Self::Digit;

    fn num_digits(self) -> DigitIndexInt;

    fn sort(original: &mut [Self])
    where [(); RADIX as usize]:
    {
        let original_ptr: *const Self = original.as_ptr();

        let mut other_vec = original.to_vec();
        let mut bucket_sizes = [0; RADIX as usize];
        let mut vec1 = &mut *original;
        let mut vec2 = &mut other_vec[..];
        let mut max_digit_num = 0;
        for x in vec1.iter().copied() {
            max_digit_num = max_digit_num.max(x.num_digits());
        }
        for i in 0..max_digit_num {
            // Look at the i-th digit of each item and count how many items we're going to put in
            // each bucket.
            bucket_sizes.fill(0);
            for x in vec1.iter().copied() {
                let digit: RadixInt = x.digit(i).into();
                bucket_sizes[digit as usize] += 1;
            }

            // Instead of the size of each bucket, save the starting index of each bucket.
            // After that loop `bucket_sizes` actually holds for each bucket the next free index
            // in that bucket.
            let mut next_bucket_index = 0;
            for i in 0..(RADIX as usize) {
                let size = bucket_sizes[i];
                bucket_sizes[i] = next_bucket_index;
                next_bucket_index += size;
            }

            for item in vec1.iter().copied() {
                let digit: usize = item.digit(i).into().into();
                let index = bucket_sizes[digit];
                bucket_sizes[digit] += 1;
                vec2[index] = item;
            }
            mem::swap(&mut vec1, &mut vec2);
        }
        if std::ptr::eq(vec1.as_ptr(), original_ptr).not() {
            vec2.copy_from_slice(vec1);
        }
    }
}

/// For when the radix is a power of two.
#[inline]
fn get_power_of_two_radix_digit<I>(n: I, radix: RadixInt, i: DigitIndexInt) -> <<I as std::ops::Shl<u8>>::Output as std::ops::Shr<u8>>::Output
where
    I: Integer + std::ops::Shl<u8>,
    <I as std::ops::Shl<u8>>::Output: std::ops::Shr<u8>,
{
    debug_assert!(radix.is_power_of_two());

    // Since radix is a power of two, n can be broken into groups of `bits_in_digit` bits; each
    // such bit group is one digit.
    // For example, if radix is 4, then a number can be divided to bit pairs, starting from the
    // right side, each bit pair is 1 digit.
    // So there are 2 bits_in_digit in that case.
    // And we get that number by taking the trailing zeros in the binary representation of 4: 0b100
    // because that's how many bits you need to represent 4 options.
    // That's always the case cause 2^x is 1 followed by x zeros, and x bits give you 2^x options.
    let bits_in_digit = radix.trailing_zeros() as u8;

    // 8 in u8, 16 in u16, 32 in u32...
    let bits_in_int = size_of::<I>() as u8 * 8;

    // if this is the number divided to digits:
    // [d][d]...[d]
    // and we want the i'th digit,
    // this variable holds the number of bits in this portion of the number:
    //        i     1  0
    // [d]...[d]...[d][d]
    //          |<----->|
    let bits_right_to_digit_originally = i * bits_in_digit;

    //        i     1  0
    // [d]...[d]...[d][d]
    //       |<-------->|
    // for this number to be correct, a digit can't have more bits than the integer.
    // But it wouldn't make sense to choose a radix that way anyway, cause then there are more
    // buckets than possible numbers, and the sorting is very inefficient.
    //
    // The min is for when we're looking at the leftmost digit and it doesn't completely fit in the
    // integer. For example, when `I` is `u8`, and radix is 8. Each digit has 3 bits. When looking
    // at the digit in index 2, meaning the leftmost digit, only 2 bits fit in the integer.
    //  bbbbbbbb
    // [d][d][d]
    let bits_in_digit_and_right = bits_in_int.min(bits_right_to_digit_originally + bits_in_digit);

    //        i     1  0
    // [d]...[d]...[d][d]
    // |<-->|
    let bits_left_to_digit = bits_in_int - bits_in_digit_and_right;

    //  i     1  0              <- original index
    //  n                 0     <- new index
    // [d]...[d][d][0]...[0]
    let with_digit_at_left = n << bits_left_to_digit;

    //  i     1  0              <- original index
    //  n                 0     <- new index
    // [d]...[d][d][0]...[0]
    //    |<-------------->|
    //
    //
    let bits_right_to_digit = bits_right_to_digit_originally + bits_left_to_digit;

    with_digit_at_left >> bits_right_to_digit
}

macro_rules! impl_copy_radixed_for_integer {
    ( $t:ty, $radix:expr ) => {
        impl RadixedCopy<$radix> for $t {
            type Digit = u8;
            fn digit(self, i: DigitIndexInt) -> Self::Digit {
                get_power_of_two_radix_digit(self, $radix, i) as Self::Digit
            }
            fn num_digits(self) -> DigitIndexInt {
                // How many bits are there in one "digit" - e.g. if radix is 16, there are 4 bits
                // in one digit.
                let bits_in_digit = $radix.trailing_zeros() as usize;
                let bits_in_int = size_of::<Self>() * 8;
                let bits_in_self = bits_in_int - self.leading_zeros() as usize;
                // This -1,+1 is to get a rounded up division.
                // Example, if there are 3 bits in a digit (radix is 8), but the number `self` only
                // takes up 1 bit: 1/3 is 0, but there is actually 1 digit in self.
                // We need round up division:
                // (1 - 1) / 3 + 1 = 1.
                // That way when the number of bits isn't divisible by the number of digits, we get
                // another digit for the division rest.
                let num_digits = (bits_in_self.saturating_sub(1)) / bits_in_digit + 1;
                num_digits as DigitIndexInt
            }
        }
    };
}

impl_copy_radixed_for_integer!(u8, 2u8);
impl_copy_radixed_for_integer!(u8, 4u8);
impl_copy_radixed_for_integer!(u8, 8u8);
impl_copy_radixed_for_integer!(u8, 16u8);
impl_copy_radixed_for_integer!(u8, 32u8);
impl_copy_radixed_for_integer!(u8, 64u8);
impl_copy_radixed_for_integer!(u16, 2u8);
impl_copy_radixed_for_integer!(u16, 4u8);
impl_copy_radixed_for_integer!(u16, 8u8);
impl_copy_radixed_for_integer!(u16, 16u8);
impl_copy_radixed_for_integer!(u16, 32u8);
impl_copy_radixed_for_integer!(u16, 64u8);
impl_copy_radixed_for_integer!(u32, 2u8);
impl_copy_radixed_for_integer!(u32, 4u8);
impl_copy_radixed_for_integer!(u32, 8u8);
impl_copy_radixed_for_integer!(u32, 16u8);
impl_copy_radixed_for_integer!(u32, 32u8);
impl_copy_radixed_for_integer!(u32, 64u8);
impl_copy_radixed_for_integer!(u64, 2u8);
impl_copy_radixed_for_integer!(u64, 4u8);
impl_copy_radixed_for_integer!(u64, 8u8);
impl_copy_radixed_for_integer!(u64, 16u8);
impl_copy_radixed_for_integer!(u64, 32u8);
impl_copy_radixed_for_integer!(u64, 64u8);



#[cfg(test)]
mod tests {
    use rstest::rstest;
    use paste::paste;
    use super::*;

    #[test]
    fn radix_digit() {
        assert_eq!(get_power_of_two_radix_digit(0u8, 2, 0), 0);
        assert_eq!(get_power_of_two_radix_digit(0u32, 2, 0), 0);
        assert_eq!(get_power_of_two_radix_digit(0u64, 2, 0), 0);

        assert_eq!(get_power_of_two_radix_digit(1u8, 2, 0), 1);
        assert_eq!(get_power_of_two_radix_digit(1u32, 2, 0), 1);
        assert_eq!(get_power_of_two_radix_digit(1u64, 2, 0), 1);

        assert_eq!(get_power_of_two_radix_digit(0b10u8, 2, 0), 0);
        assert_eq!(get_power_of_two_radix_digit(0b10u32, 2, 0), 0);
        assert_eq!(get_power_of_two_radix_digit(0b10u64, 2, 0), 0);

        assert_eq!(get_power_of_two_radix_digit(0b10u8, 2, 1), 1);
        assert_eq!(get_power_of_two_radix_digit(0b10u32, 2, 1), 1);
        assert_eq!(get_power_of_two_radix_digit(0b10u64, 2, 1), 1);

        assert_eq!(get_power_of_two_radix_digit(0x10u8, 16, 0), 0);
        assert_eq!(get_power_of_two_radix_digit(0x10u32, 16, 0), 0);
        assert_eq!(get_power_of_two_radix_digit(0x10u64, 16, 0), 0);

        assert_eq!(get_power_of_two_radix_digit(0x10u8, 16, 1), 1);
        assert_eq!(get_power_of_two_radix_digit(0x10u32, 16, 1), 1);
        assert_eq!(get_power_of_two_radix_digit(0x10u64, 16, 1), 1);

        assert_eq!(get_power_of_two_radix_digit(0x30u8, 16, 1), 3);
        assert_eq!(get_power_of_two_radix_digit(0x30u32, 16, 1), 3);
        assert_eq!(get_power_of_two_radix_digit(0x30u64, 16, 1), 3);

    }

    #[rstest]
    fn sort_u32s_with_radix_4(
        #[values(
            vec![3, 4, 9, 2, 1, 0, 3],
            vec![1, 2, 3],
            vec![0, 0, 0],
            vec![3, 2, 1],
            vec![100000000, 1000, 1, 10],
        )]
        mut vec: Vec<u32>
    ) {
        let mut vec2 = vec.clone();
        RadixedCopy::<4>::sort(&mut vec);
        vec2.sort();
        assert_eq!(vec, vec2);
    }

    macro_rules! test_sort_type_with_radix {
        ( $t:ty, $radix:expr ) => {
            paste! {
                #[rstest]
                fn [<sort_ $t:lower _with_radix_ $radix >](
                    #[values(
                        vec![3, 4, 9, 2, 1, 0, 3],
                        vec![1, 2, 3],
                        vec![0, 0, 0],
                        vec![3, 2, 1],
                        vec![[<100000000 $t>], [<1000 $t>], 1, 10],
                    )]
                    mut vec: Vec<$t>
                ) {
                    let mut vec2 = vec.clone();
                    RadixedCopy::<$radix>::sort(&mut vec);
                    vec2.sort();
                    assert_eq!(vec, vec2);
                }
            }
        }
    }

    test_sort_type_with_radix!(u8, 2);
    test_sort_type_with_radix!(u16, 2);
    test_sort_type_with_radix!(u32, 2);
    test_sort_type_with_radix!(u64, 2);
    test_sort_type_with_radix!(u8, 4);
    test_sort_type_with_radix!(u16, 4);
    test_sort_type_with_radix!(u32, 4);
    test_sort_type_with_radix!(u64, 4);
    test_sort_type_with_radix!(u8, 8);
    test_sort_type_with_radix!(u16, 8);
    test_sort_type_with_radix!(u32, 8);
    test_sort_type_with_radix!(u64, 8);
    test_sort_type_with_radix!(u8, 16);
    test_sort_type_with_radix!(u16, 16);
    test_sort_type_with_radix!(u32, 16);
    test_sort_type_with_radix!(u64, 16);
    test_sort_type_with_radix!(u8, 32);
    test_sort_type_with_radix!(u16, 32);
    test_sort_type_with_radix!(u32, 32);
    test_sort_type_with_radix!(u64, 32);
}
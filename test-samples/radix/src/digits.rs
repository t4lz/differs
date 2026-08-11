use crate::{DigitIndexInt, RadixInt};
use num_integer::Integer;

/// For when the radix is a power of two.
#[inline]
pub(crate) fn get_power_of_two_radix_digit<I>(n: I, radix: RadixInt, i: DigitIndexInt) -> <<I as std::ops::Shl<u8>>::Output as std::ops::Shr<u8>>::Output
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
    let bits_in_digit_and_right = bits_in_int.max(bits_right_to_digit_originally + bits_in_digit);

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

# differs

A Rust and Git aware diff command. Compare a single Rust item across files, modules and git refs.

Sometimes a code section is moved and editted in the same commit. That's not the best practice,
but it happens. When that happens, it's harder to see what actually changed, since the normal diff
shows everything that moved as changed.
Differs compares items by name/path across different files and git refs.


## Installation

```bash
cargo build             # builds target/debug/differs
cargo install --path .  # or put `differs` on your PATH
```

## Example

This repository carries the demo on two branches. `example-before` has a radix sort with a
65-line function `get_power_of_two_radix_digit` in `lib.rs`. `example-after` moves it to a new
file, `digits.rs`, and changes one call inside it.

```bash
git switch example-after
```

Git's view of that - 138 lines across two files. The unexpected one-line changed is well-hidden within diff:

```bash
git diff example-before example-after -- test-samples/radix/
```

```diff
diff --git a/test-samples/radix/src/digits.rs b/test-samples/radix/src/digits.rs
new file mode 100644
index 0000000..0c81e09
--- /dev/null
+++ b/test-samples/radix/src/digits.rs
@@ -0,0 +1,68 @@
+use crate::{DigitIndexInt, RadixInt};
+use num_integer::Integer;
+
+/// For when the radix is a power of two.
+#[inline]
+pub(crate) fn get_power_of_two_radix_digit<I>(n: I, radix: RadixInt, i: DigitIndexInt) -> <<I as std::ops::Shl<u8>>::Output as std::ops::Shr<u8>>::Output
+where
+    I: Integer + std::ops::Shl<u8>,
+    <I as std::ops::Shl<u8>>::Output: std::ops::Shr<u8>,
+{
+    debug_assert!(radix.is_power_of_two());
+
+    // Since radix is a power of two, n can be broken into groups of `bits_in_digit` bits; each
+    // such bit group is one digit.
+    // For example, if radix is 4, then a number can be divided to bit pairs, starting from the
+    // right side, each bit pair is 1 digit.
+    // So there are 2 bits_in_digit in that case.
+    // And we get that number by taking the trailing zeros in the binary representation of 4: 0b100
+    // because that's how many bits you need to represent 4 options.
+    // That's always the case cause 2^x is 1 followed by x zeros, and x bits give you 2^x options.
+    let bits_in_digit = radix.trailing_zeros() as u8;
+
+    // 8 in u8, 16 in u16, 32 in u32...
+    let bits_in_int = size_of::<I>() as u8 * 8;
+
+    // if this is the number divided to digits:
+    // [d][d]...[d]
+    // and we want the i'th digit,
+    // this variable holds the number of bits in this portion of the number:
+    //        i     1  0
+    // [d]...[d]...[d][d]
+    //          |<----->|
+    let bits_right_to_digit_originally = i * bits_in_digit;
+
+    //        i     1  0
+    // [d]...[d]...[d][d]
+    //       |<-------->|
+    // for this number to be correct, a digit can't have more bits than the integer.
+    // But it wouldn't make sense to choose a radix that way anyway, cause then there are more
+    // buckets than possible numbers, and the sorting is very inefficient.
+    //
+    // The min is for when we're looking at the leftmost digit and it doesn't completely fit in the
+    // integer. For example, when `I` is `u8`, and radix is 8. Each digit has 3 bits. When looking
+    // at the digit in index 2, meaning the leftmost digit, only 2 bits fit in the integer.
+    //  bbbbbbbb
+    // [d][d][d]
+    let bits_in_digit_and_right = bits_in_int.max(bits_right_to_digit_originally + bits_in_digit);
+
+    //        i     1  0
+    // [d]...[d]...[d][d]
+    // |<-->|
+    let bits_left_to_digit = bits_in_int - bits_in_digit_and_right;
+
+    //  i     1  0              <- original index
+    //  n                 0     <- new index
+    // [d]...[d][d][0]...[0]
+    let with_digit_at_left = n << bits_left_to_digit;
+
+    //  i     1  0              <- original index
+    //  n                 0     <- new index
+    // [d]...[d][d][0]...[0]
+    //    |<-------------->|
+    //
+    //
+    let bits_right_to_digit = bits_right_to_digit_originally + bits_left_to_digit;
+
+    with_digit_at_left >> bits_right_to_digit
+}
diff --git a/test-samples/radix/src/lib.rs b/test-samples/radix/src/lib.rs
index 041184a..963f0f4 100644
--- a/test-samples/radix/src/lib.rs
+++ b/test-samples/radix/src/lib.rs
@@ -1,8 +1,10 @@
 #![feature(generic_const_exprs)]
 
+mod digits;
+
 use std::mem;
-use num_integer::Integer;
 use std::ops::Not;
+use digits::get_power_of_two_radix_digit;
 
 /// Integer that can describe number of buckets, and can be the index of a bucket.
 type RadixInt = u8;
@@ -79,72 +81,6 @@ trait RadixedCopy<const RADIX: RadixInt>: Copy {
     }
 }
 
-/// For when the radix is a power of two.
-#[inline]
-fn get_power_of_two_radix_digit<I>(n: I, radix: RadixInt, i: DigitIndexInt) -> <<I as std::ops::Shl<u8>>::Output as std::ops::Shr<u8>>::Output
-where
-    I: Integer + std::ops::Shl<u8>,
-    <I as std::ops::Shl<u8>>::Output: std::ops::Shr<u8>,
-{
-    debug_assert!(radix.is_power_of_two());
-
-    // Since radix is a power of two, n can be broken into groups of `bits_in_digit` bits; each
-    // such bit group is one digit.
-    // For example, if radix is 4, then a number can be divided to bit pairs, starting from the
-    // right side, each bit pair is 1 digit.
-    // So there are 2 bits_in_digit in that case.
-    // And we get that number by taking the trailing zeros in the binary representation of 4: 0b100
-    // because that's how many bits you need to represent 4 options.
-    // That's always the case cause 2^x is 1 followed by x zeros, and x bits give you 2^x options.
-    let bits_in_digit = radix.trailing_zeros() as u8;
-
-    // 8 in u8, 16 in u16, 32 in u32...
-    let bits_in_int = size_of::<I>() as u8 * 8;
-
-    // if this is the number divided to digits:
-    // [d][d]...[d]
-    // and we want the i'th digit,
-    // this variable holds the number of bits in this portion of the number:
-    //        i     1  0
-    // [d]...[d]...[d][d]
-    //          |<----->|
-    let bits_right_to_digit_originally = i * bits_in_digit;
-
-    //        i     1  0
-    // [d]...[d]...[d][d]
-    //       |<-------->|
-    // for this number to be correct, a digit can't have more bits than the integer.
-    // But it wouldn't make sense to choose a radix that way anyway, cause then there are more
-    // buckets than possible numbers, and the sorting is very inefficient.
-    //
-    // The min is for when we're looking at the leftmost digit and it doesn't completely fit in the
-    // integer. For example, when `I` is `u8`, and radix is 8. Each digit has 3 bits. When looking
-    // at the digit in index 2, meaning the leftmost digit, only 2 bits fit in the integer.
-    //  bbbbbbbb
-    // [d][d][d]
-    let bits_in_digit_and_right = bits_in_int.min(bits_right_to_digit_originally + bits_in_digit);
-
-    //        i     1  0
-    // [d]...[d]...[d][d]
-    // |<-->|
-    let bits_left_to_digit = bits_in_int - bits_in_digit_and_right;
-
-    //  i     1  0              <- original index
-    //  n                 0     <- new index
-    // [d]...[d][d][0]...[0]
-    let with_digit_at_left = n << bits_left_to_digit;
-
-    //  i     1  0              <- original index
-    //  n                 0     <- new index
-    // [d]...[d][d][0]...[0]
-    //    |<-------------->|
-    //
-    //
-    let bits_right_to_digit = bits_right_to_digit_originally + bits_left_to_digit;
-
-    with_digit_at_left >> bits_right_to_digit
-}
-
 macro_rules! impl_copy_radixed_for_integer {
     ( $t:ty, $radix:expr ) => {
         impl RadixedCopy<$radix> for $t {
```

It's hard to know what changed. Now ask about the function itself - old path on
the left, new path on the right:

```bash
differs example-before:test-samples/radix/src/lib.rs   get_power_of_two_radix_digit \
        example-after:test-samples/radix/src/digits.rs get_power_of_two_radix_digit
```

```diff
 /// For when the radix is a power of two.
 #[inline]
-fn get_power_of_two_radix_digit<I>(n: I, radix: RadixInt, i: DigitIndexInt) -> ...
+pub(crate) fn get_power_of_two_radix_digit<I>(n: I, radix: RadixInt, i: DigitIndexInt) -> ...
 where
     I: Integer + std::ops::Shl<u8>,
     <I as std::ops::Shl<u8>>::Output: std::ops::Shr<u8>,
     // at the digit in index 2, meaning the leftmost digit, only 2 bits fit in the integer.
     //  bbbbbbbb
     // [d][d][d]
-    let bits_in_digit_and_right = bits_in_int.min(bits_right_to_digit_originally + bits_in_digit);
+    let bits_in_digit_and_right = bits_in_int.max(bits_right_to_digit_originally + bits_in_digit);

     //        i     1  0
     // [d]...[d]...[d][d]
```

Two lines. One is the visibility bump the move required; the other is a `min` that became a `max`
in the middle of the bit arithmetic, which breaks the sort algorithm.

## Usage

```
differs <OLD_FILE> <OLD_ITEM_PATH> <NEW_FILE> <NEW_ITEM_PATH>
```

Each file argument is a path, optionally prefixed with a git ref and a colon.

Each item path names one item within that file - `my_mod::MyStruct::new`, or just `new` to search
the whole file by name. The two sides are independent - the file, the module
path and the name may all differ between them.

Item lookup is [hol](https://github.com/t4lz/hol), which resolves the path with rust-analyzer and
returns the item's source including its attributes and doc comments.

## Known limitations

Both come from item lookup and are tracked upstream:

- **Every path you name must also exist in your working tree**, even when you have given it a git
  ref ([hol#1](https://github.com/t4lz/hol/issues/1)). The ref chooses which version to read; it
  can't make the path resolvable. So if a file was renamed outright you cannot ask for the old
  side from the new branch. It is also why the example above starts with `git switch` - on `main`
  there is no `test-samples/`.
- **Run from the repository root, or pass absolute paths**
  ([hol#2](https://github.com/t4lz/hol/issues/2)). A relative path is interpreted against your
  current directory when locating the repo, but against the repo root when reading from the tree.

## License

MIT or Apache-2.0, at your option.

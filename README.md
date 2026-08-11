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

Git's view of that — 138 lines across two files, one of them brand new:

```bash
git diff --stat example-before -- test-samples/radix/
```

```
 test-samples/radix/src/digits.rs | 68 ++++++++++++++++++++++++++++++++++++++
 test-samples/radix/src/lib.rs    | 70 ++--------------------------------------
 2 files changed, 71 insertions(+), 67 deletions(-)
```

It's hard to know what changed. Now ask about the function itself — old path on
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
in the middle of the bit arithmetic, which breaks the sort for every input.

## Usage

```
differs <OLD_FILE> <OLD_ITEM_PATH> <NEW_FILE> <NEW_ITEM_PATH>
```

Each file argument is a path, optionally prefixed with a git ref and a colon.

Each item path names one item within that file — `my_mod::MyStruct::new`, or just `new` to search
the whole file by name. The two sides are independent - the file, the module
path and the name may all differ between them.

Item lookup is [hol](https://github.com/t4lz/hol), which resolves the path with rust-analyzer and
returns the item's source including its attributes and doc comments.

## Known limitations

Both come from item lookup and are tracked upstream:

- **Every path you name must also exist in your working tree**, even when you have given it a git
  ref ([hol#1](https://github.com/t4lz/hol/issues/1)). The ref chooses which version to read; it
  can't make the path resolvable. So if a file was renamed outright you cannot ask for the old
  side from the new branch. It is also why the example above starts with `git switch` — on `main`
  there is no `test-samples/`.
- **Run from the repository root, or pass absolute paths**
  ([hol#2](https://github.com/t4lz/hol/issues/2)). A relative path is interpreted against your
  current directory when locating the repo, but against the repo root when reading from the tree.

## License

MIT or Apache-2.0, at your option.

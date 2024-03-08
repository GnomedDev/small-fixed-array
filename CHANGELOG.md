# 0.4.1

- Implements `clone_from` for FixedString and FixedArray.

# 0.4

- Removes the `Index<usize>` implementation from `FixedArray`.
- Implements a `&'static str` variant of `FixedString` created with `FixedString::from_str_static`.

# 0.3

- Returns LenT from `FixedArray::len` and `FixedString::len`, instead of `u32`.
- Implements indexing a `FixedArray` with `LenT`, alongside the `usize` indexing implementation.
- Implements no-std support, with an optional `std` feature enabled by default.
- Implements `Borrow<str>` for `FixedString`, allowing a `HashMap` of `FixedString` to be looked up via `&str`.

# 0.2.1

- Improves FixedString to gain another SSO inline character. This comes at a tiny CPU performance hit that can be counteracted with the `nightly` feature.
- Improves FixedString deserialisation to take advantage of SSO more.
- Implements `From<[T]; 0..=16>` for FixedArray.

# 0.2

- Redesigns the API to make truncation explicit, either via `from_*_trunc` or `trunc_into`.

# 0.1.3

- Implements `From<Cow<'_, str/[T]>>` for FixedString/Array, returning Owned Cows.
- Implements `TryFrom<Box<str>>` for FixedString, to allow constructing it without logging features.
- Implements SSO for FixedString, reducing allocations and shrinking memory usage.
- Reduces compile time more, with more logic split out of generics.

# 0.1.2

- Reduces compile time by splitting logic out of generics.
- Attempts to fix documentation on docs.rs not showing features.

# 0.1.1

- Fix compilation with only serde feature enabled.

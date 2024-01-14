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

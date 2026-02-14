# ptest

Procedural macros for parameterized testing in Rust. Generate multiple test functions from a single test template using values, types, or a combination of both.

## Macros

### `#[value_parameterized_test]`

Generates a test for each provided value. The annotated function must have exactly one argument.

- **`args`** — array of expressions; test names are derived from the expression.
- **`named_args`** — array of `("name", expr)` tuples; the string is used as the test name suffix.

```rust
use ptest::value_parameterized_test;

#[value_parameterized_test(args = [1, 2, 3])]
fn test_is_positive(x: i32) {
    assert!(x > 0);
}

#[value_parameterized_test(named_args = [("small", 1), ("large", 1000)])]
fn test_is_positive(x: i32) {
    assert!(x > 0);
}
```

### `#[type_parameterized_test]`

Generates a test for each provided type. The annotated function must have zero arguments and one generic type parameter.

```rust
use ptest::type_parameterized_test;

#[type_parameterized_test(types = (u32, i32, f64))]
fn test_default<T: Default + std::fmt::Debug>() {
    let value = T::default();
    println!("{:?}", value);
}
```

### `#[matrix_parameterized_test]`

Generates a test for every combination of types and arguments (cartesian product). The annotated function must be generic over one type parameter and accept exactly one argument.

```rust
use ptest::matrix_parameterized_test;

#[matrix_parameterized_test(
    types = (Vec<i32>, VecDeque<i32>),
    named_args = [("small", build_small()), ("large", build_large())],
)]
fn test_len<T: Collection>(c: &T) {
    assert!(c.len() > 0);
}
```

## License

See [LICENSE](LICENSE) for details.

---

*This README was generated with the assistance of AI. AI was used solely for documentation purposes — all library code was written by hand.*

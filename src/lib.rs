use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{ExprArray, TypeTuple};

#[derive(deluxe::ParseMetaItem)]
struct ValueParameterizedTestAttribute {
    args: Option<ExprArray>,
    named_args: Option<ExprArray>,
}

/// ## `value_parameterized_test` Attribute Proc Macro
///
/// Generates multiple test functions from a single test template by specifying a list of values.
/// Each generated test calls the original function with one of the provided values as its argument.
///
/// ### Parameters
///
/// - `args` (optional): An array of expressions passed directly as arguments. Test names are derived from the expression.
/// - `named_args` (optional): An array of `("name", expr)` tuples. The string is used for the test name and `expr` is passed as the argument.
///
/// The annotated function must have exactly one argument.
///
/// ### Examples
///
/// Using `args`:
/// ```rust
/// #[value_parameterized_test(args = [1, 2, 3])]
/// fn test_is_positive(x: i32) {
///     assert!(x > 0);
/// }
/// ```
///
/// This will generate three test functions:
/// - `test_is_positive_1`
/// - `test_is_positive_2`
/// - `test_is_positive_3`
///
/// Using `named_args`:
/// ```rust
/// #[value_parameterized_test(named_args = [("small", 1), ("large", 1000)])]
/// fn test_is_positive(x: i32) {
///     assert!(x > 0);
/// }
/// ```
///
/// This will generate:
/// - `test_is_positive_small`
/// - `test_is_positive_large`
///
/// Both `args` and `named_args` can be combined in a single invocation.
#[proc_macro_attribute]
pub fn value_parameterized_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let ValueParameterizedTestAttribute { args, named_args } =
        deluxe::parse::<ValueParameterizedTestAttribute>(attr)
            .expect("Failed to parse ValueParameterizedTestAttribute");

    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let item_attrs = &item_fn.attrs;
    let item_vis = &item_fn.vis;
    let item_sig = &item_fn.sig;
    let item_block = &item_fn.block;

    let item_ident = &item_sig.ident;
    let _item_inputs = &item_sig.inputs;

    if item_sig.inputs.len() != 1usize {
        panic!("Function must have exactly one argument");
    }

    let default_args = ExprArray {
        attrs: Vec::new(),
        bracket_token: syn::token::Bracket::default(),
        elems: syn::punctuated::Punctuated::new(),
    };

    // Build (name, arg_expr) pairs from both args and named_args
    let unnamed: Vec<(String, &dyn ToTokens)> = args
        .as_ref()
        .unwrap_or_else(|| &default_args)
        .elems
        .iter()
        .map(|arg| {
            let mut s = arg.to_token_stream().to_string();
            s = s.replace(
                ['"', '\'', '.', '-', '[', ']', '(', ')', '{', '}', ','],
                "_",
            );
            s = s.replace(' ', "_");
            (s, arg as &dyn ToTokens)
        })
        .collect();

    let named_parsed: Vec<(String, Box<dyn ToTokens>)> = named_args
        .as_ref()
        .unwrap_or_else(|| &default_args)
        .elems
        .iter()
        .map(|elem| {
            let tuple = match elem {
                syn::Expr::Tuple(t) => t.clone(),
                _ => panic!("named_args elements must be tuples like (\"name\", expr)"),
            };
            let args = tuple.elems[1].clone();
            let name = match &tuple.elems[0] {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => s.value(),
                _ => panic!("First element of named_args tuple must be a string literal"),
            };
            (name, Box::new(args) as Box<dyn ToTokens>)
        })
        .collect();

    let named: Vec<(String, &dyn ToTokens)> = named_parsed
        .iter()
        .map(|(name, tuple)| (name.clone(), &**tuple as &dyn ToTokens))
        .collect();

    let all_args: Vec<(String, &dyn ToTokens)> =
        unnamed.iter().chain(named.iter()).cloned().collect();

    let test_defs = all_args.iter().map(move |(arg_name, arg_expr)| {
        let arg_name = arg_name.replace(
            ['"', '\'', '.', '-', '[', ']', '(', ')', '{', '}', ',', ' '],
            "_",
        );

        let test_name = syn::Ident::new(
            &format!("{}_{}", item_ident, arg_name.to_case(Case::Snake)),
            item_ident.span(),
        );

        quote! {
            #[test]
            #(#item_attrs)*
            fn #test_name() {
                #item_ident(#arg_expr)
            }
        }
    });

    quote! {
        #item_vis #item_sig #item_block

        #(#test_defs)*
    }
    .into()
}

#[derive(deluxe::ParseMetaItem)]
struct TypeParameterizedTestAttribute {
    types: TypeTuple,
}

/// ## `type_parameterized_test` Attribute Proc Macro
///
/// Generates multiple test functions from a single generic test template by specifying a list of types.
/// Each generated test calls the original function with one of the provided types as its generic parameter.
///
/// ### Parameters
///
/// - `types`: A tuple of types to parameterize over.
///
/// The annotated function must have zero arguments and be generic over one type parameter.
///
/// ### Example
///
/// ```rust
/// #[type_parameterized_test(types = (u32, i32, f64))]
/// fn test_default<T: Default + std::fmt::Debug>() {
///     let value = T::default();
///     println!("{:?}", value);
/// }
/// ```
///
/// This will generate three test functions:
/// - `test_default_u32`
/// - `test_default_i32`
/// - `test_default_f64`
///
/// Each calls `test_default` with the corresponding type as its generic parameter.
#[proc_macro_attribute]
pub fn type_parameterized_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let TypeParameterizedTestAttribute { types: values } =
        deluxe::parse::<TypeParameterizedTestAttribute>(attr)
            .expect("Failed to parse TypeParameterizedTestAttribute");

    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let item_attrs = &item_fn.attrs;
    let item_vis = &item_fn.vis;
    let item_sig = &item_fn.sig;
    let item_block = &item_fn.block;

    let item_ident = &item_sig.ident;
    let _item_inputs = &item_sig.inputs;

    if item_sig.inputs.len() != 0 {
        panic!("Function must have exactly zero arguments");
    }

    let test_defs = values.elems.iter().map(|val| {
        let test_name = syn::Ident::new(
            &format!(
                "{}_{}",
                item_ident,
                val.to_token_stream().to_string().to_case(Case::Snake)
            ),
            item_ident.span(),
        );
        quote! {
            #[test]
            #(#item_attrs)*
            fn #test_name() {
                #item_ident::<#val>()
            }
        }
    });

    quote! {
        #item_vis #item_sig #item_block

        #(#test_defs)*
    }
    .into()
}

#[derive(deluxe::ParseMetaItem)]
struct MatrixParameterizedTestAttribute {
    types: TypeTuple,
    args: Option<ExprArray>,
    named_args: Option<ExprArray>,
}

/// ## `matrix_parameterized_test` Attribute Proc Macro
///
/// Generates a test for each combination of types and arguments (cartesian product).
/// The annotated function must be generic over one type parameter and accept exactly one argument.
///
/// ### Parameters
///
/// - `types`: A tuple of types to parameterize over.
/// - `args` (optional): An array of expressions passed directly as arguments. Test names are derived from the expression.
/// - `named_args` (optional): An array of `("name", expr)` tuples. The string is used for the test name and `expr` is passed as the argument.
///
/// ### Examples
///
/// Using `args`:
/// ```rust
/// #[matrix_parameterized_test(
///     types = (Vec<i32>, VecDeque<i32>),
///     args = [build_small(), build_large()],
/// )]
/// fn test_len<T: Collection>(c: &T) {
///     assert!(c.len() > 0);
/// }
/// ```
///
/// Using `named_args`:
/// ```rust
/// #[matrix_parameterized_test(
///     types = (Vec<i32>, VecDeque<i32>),
///     named_args = [("small", build_small()), ("large", build_large())],
/// )]
/// fn test_len<T: Collection>(c: &T) {
///     assert!(c.len() > 0);
/// }
/// ```
///
/// Both generate tests like `test_len_vec_i_32_small`, `test_len_vec_i_32_large`, etc.
#[proc_macro_attribute]
pub fn matrix_parameterized_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let MatrixParameterizedTestAttribute {
        types,
        args,
        named_args,
    } = deluxe::parse::<MatrixParameterizedTestAttribute>(attr)
        .expect("Failed to parse MatrixParameterizedTestAttribute");

    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let item_attrs = &item_fn.attrs;
    let item_vis = &item_fn.vis;
    let item_sig = &item_fn.sig;
    let item_block = &item_fn.block;

    let item_ident = &item_sig.ident;
    let _item_inputs = &item_sig.inputs;

    if item_sig.inputs.len() != 1usize {
        panic!("Function must have exactly one argument");
    }

    let default_args = ExprArray {
        attrs: Vec::new(),
        bracket_token: syn::token::Bracket::default(),
        elems: syn::punctuated::Punctuated::new(),
    };

    // Build (name, arg_expr) pairs from both args and named_args
    let unnamed: Vec<(String, &dyn ToTokens)> = args
        .as_ref()
        .unwrap_or_else(|| &default_args)
        .elems
        .iter()
        .map(|arg| {
            let mut s = arg.to_token_stream().to_string();
            s = s.replace(
                ['"', '\'', '.', '-', '[', ']', '(', ')', '{', '}', ','],
                "_",
            );
            s = s.replace(' ', "_");
            (s, arg as &dyn ToTokens)
        })
        .collect();

    let named_parsed: Vec<(String, Box<dyn ToTokens>)> = named_args
        .as_ref()
        .unwrap_or_else(|| &default_args)
        .elems
        .iter()
        .map(|elem| {
            let tuple = match elem {
                syn::Expr::Tuple(t) => t.clone(),
                _ => panic!("named_args elements must be tuples like (\"name\", expr)"),
            };
            let args = tuple.elems[1].clone();
            let name = match &tuple.elems[0] {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => s.value(),
                _ => panic!("First element of named_args tuple must be a string literal"),
            };
            (name, Box::new(args) as Box<dyn ToTokens>)
        })
        .collect();

    let named: Vec<(String, &dyn ToTokens)> = named_parsed
        .iter()
        .map(|(name, tuple)| (name.clone(), &**tuple as &dyn ToTokens))
        .collect();

    let all_args: Vec<(String, &dyn ToTokens)> =
        unnamed.iter().chain(named.iter()).cloned().collect();

    let test_defs = types.elems.iter().flat_map(|ty| {
        let ty_name = ty.to_token_stream().to_string().to_case(Case::Snake);
        all_args.iter().map(move |(arg_name, arg_expr)| {
            let test_name = syn::Ident::new(
                &format!(
                    "{}_{}_{}",
                    item_ident,
                    ty_name,
                    arg_name.to_case(Case::Snake)
                ),
                item_ident.span(),
            );
            quote! {
                #[test]
                #(#item_attrs)*
                fn #test_name() {
                    #item_ident::<#ty>(#arg_expr)
                }
            }
        })
    });

    quote! {
        #item_vis #item_sig #item_block

        #(#test_defs)*
    }
    .into()
}

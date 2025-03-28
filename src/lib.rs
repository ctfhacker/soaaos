use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Ident, Lifetime, LifetimeParam, LitStr,
    parse_macro_input, spanned::Spanned,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Layout {
    StructOfArrays,
    ArrayOfStructs,
}

/// Implement a Struct-of-Arrays or Array-of-Structs collection of a single struct
///
/// Example:
///
/// ```rust
/// use core::error::Error;
/// /// The struct `NodesLayout` is created as a `struct-of-arrays`
/// #[soaaos::layout("struct-of-arrays")]
/// // #[layout("aos")] // For Array-of-Structs
/// struct Node<R> where R: std::fmt::Debug + PartialEq {
///   name: String,
///   operation: u8,
///   arg2: R,
/// }
///
/// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// enum Fruit {
///   Apple,
///   Banana,
/// }
///
/// fn main() {
///    let mut nodes = NodesLayout::<Fruit>::new();
///
///    nodes.add(Node { name: "Node1".to_string(), operation: 1, arg2: Fruit::Apple });
///    nodes.add(Node { name: "Node2".to_string(), operation: 2, arg2: Fruit::Banana });
///    assert_eq!(nodes.len(), 2);
///    assert_eq!(nodes.operation.iter().copied().collect::<Vec<_>>(), vec![1, 2]);
///    assert_eq!(format!("{nodes:?}"), r#"NodesLayout { name: ["Node1", "Node2"], operation: [1, 2], arg2: [Apple, Banana] }"#);
/// }
/// ```
///
/// Provides:
///
/// * `with_capacity(usize)`             - Initialize the layout with the given size for all `Vec`s
/// * `add(&mut self, node: Node)`       - Add the node to the layout
/// * `get_*(&self, id: NodeId)`         - Get `&field` of the node at the given index
/// * `get_*_mut(&mut self, id: NodeId)` - Get `&mut field` of the node at the given index
///
#[proc_macro_attribute]
pub fn layout(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input item as a DeriveInput (i.e. a struct definition).
    let input = parse_macro_input!(item as DeriveInput);

    let generics = input.generics.clone();

    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    // Parse the type of layout
    let layout;
    let text = parse_macro_input!(attr as LitStr);
    let val = text.value();
    match val.as_str() {
        "soa" | "struct-of-arrays" => layout = Layout::StructOfArrays,
        "aos" | "array-of-structs" => layout = Layout::ArrayOfStructs,
        _ => panic!(
            "Unknown memory layout (expected 'struct-of-arrays' or 'array-of-structs'): {val}"
        ),
    }

    let struct_ident = input.ident.clone();
    let struct_ident_ref = Ident::new(&format!("{}Ref", struct_ident), struct_ident.span());

    // Create the identifiers to be created
    macro_rules! new_ident {
        ($post:literal) => {
            Ident::new(&format!($post, struct_ident), struct_ident.span())
        };
    }
    let layout_struct_ident = new_ident!("{}sLayout");
    let layout_iter_ident = new_ident!("{}sIter");
    let error_ident = new_ident!("{}sError");
    let id_ident = new_ident!("{}Id");

    // Only support structs with named fields.
    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields_named) = &data.fields {
            fields_named.named.iter().collect::<Vec<_>>()
        } else {
            return syn::Error::new_spanned(
                struct_ident,
                "Only structs with named fields are supported for #[layout]",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(struct_ident, "#[layout] can only be applied to structs")
            .to_compile_error()
            .into();
    };

    // Extract the field names and types.
    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("Expected named field"))
        .collect();

    let first_field = field_names
        .get(0)
        .expect("No fields found for this memory layout");

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Create getter method names for each field (e.g. get_field1).
    let getter_names: Vec<Ident> = field_names
        .iter()
        .map(|ident| Ident::new(&format!("get_{}", ident), ident.span()))
        .collect();

    // Create getter method names for each field enumerated (e.g. get_field1_enumerated).
    let getter_enumerated_names: Vec<Ident> = field_names
        .iter()
        .map(|ident| Ident::new(&format!("get_{}_enumerated", ident), ident.span()))
        .collect();

    // Create getter mut method names for each field (e.g. get_field1_mut).
    let getter_mut_names: Vec<Ident> = field_names
        .iter()
        .map(|ident| Ident::new(&format!("get_{}_mut", ident), ident.span()))
        .collect();

    // Create getter method names for each field (e.g. get_field1).
    let error_names: Vec<Ident> = field_names
        .iter()
        .map(|ident| Ident::new(&format!("NotFound_{}", ident), ident.span()))
        .collect();

    // The ref iterator needs a lifetime prepending any given generics. Prepend a 'a lifetime to any
    // given generics.
    // <R> => <'a, R>
    let mut generics_with_lifetime = generics.clone();
    let lifetime = Lifetime::new("'a", impl_generics.span());
    generics_with_lifetime.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())),
    );

    // Same as above but for ellided lifetimes
    // <R> => <'_, R>
    let mut generics_with_ellided_lifetime = generics.clone();
    let ellided_lifetime = Lifetime::new("'_", impl_generics.span());
    generics_with_ellided_lifetime.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(ellided_lifetime.clone())),
    );

    // Create the code that is used in both struct-of-arrays and array-of-structs
    let both = quote! {
        // Keep the original struct definition.
        #input

        /// The index into the `nodes` vec
        #[allow(dead_code)]
        #[repr(transparent)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #id_ident(pub u32);
        const _: () = assert!(size_of::<#id_ident>() == 4);
        const _: () = assert!(size_of::<Option<#id_ident>>() == 8);
        const _: () = assert!(size_of::<&#id_ident>() == 8);
        impl #id_ident {
            #[must_use]
            pub fn null() -> Self {
                #id_ident(0)
            }
        }

        #[derive(Debug)]
        pub struct #struct_ident_ref #generics_with_lifetime #where_clause {
            #(
                pub #field_names: &#lifetime #field_types,
            )*
        }

        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub enum #error_ident {
            #(
                #error_names,
            )*

            InvalidDiff,
        }

        impl core::fmt::Display for #error_ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(
                        #error_ident::#error_names => write!(f, "Not Found: {}", stringify!(#error_names)),
                    )*

                    InvalidDiff => write!(f, "Invalid Diff"),
                }
            }
        }

        impl core::error::Error for #error_ident {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match self {
                    _ => None,
                }
            }
        }

        impl #impl_generics #layout_struct_ident #impl_generics #where_clause{
            /// Returns the diff (by field) between two layouts
            pub fn diff(&self, other: &Self) -> Option<String> {
                use std::fmt::Write;

                let mut out = String::new();

                #(
                    let this_iter = self.#field_names();
                    let other_iter = other.#field_names();

                    for (i, (o1, o2)) in this_iter.zip(other_iter).enumerate() {
                        if *o1 != *o2 {
                            write!(out, "\n{} {i}: {o1:?} vs {o2:?}", stringify!(#field_names)).unwrap();
                        }
                    }
                )*

                if !out.is_empty() {
                    return Some(out);
                }

                None
            }

            pub fn iter(&self) -> #layout_iter_ident #impl_generics {
                #layout_iter_ident { index: #id_ident::null(), layout: self }
            }

            pub fn iter_enumerated(&self) -> impl Iterator<Item = (#id_ident, #struct_ident_ref #generics_with_ellided_lifetime)> {
                self
                .iter()
                .enumerate()
                .map(|(index, item)| (#id_ident(index as u32), item))
            }
        }

        pub struct #layout_iter_ident #generics_with_lifetime #where_clause {
            index: #id_ident,
            layout: &'a #layout_struct_ident #impl_generics,
        }


        // Iterate through all elements in the layout, returning a struct of refs to the internal fields
        impl #generics_with_lifetime Iterator for #layout_iter_ident #generics_with_lifetime #where_clause {
            type Item = #struct_ident_ref #generics_with_lifetime;

            fn next(&mut self) -> Option<Self::Item> {
                let result = #struct_ident_ref {
                    #(
                        #field_names: self.layout.#getter_names(self.index).ok()?,
                    )*
                };

                self.index = #id_ident(self.index.0 + 1);

                Some(result)
            }
        }
    };

    // Generate different implementations based on the chosen layout.
    if layout == Layout::StructOfArrays {
        let output = quote! {
            #both

            /// Layout version using struct-of-arrays layout.
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #layout_struct_ident #impl_generics #where_clause {
                #(
                    pub #field_names: Vec<#field_types>,
                )*
            }

            impl #impl_generics #layout_struct_ident #impl_generics #where_clause {
                /// Create a new layout struct with all internal vectors initialized.
                pub fn new() -> Self {
                    // println!("Using struct-of-arrays for {}", stringify!(#struct_ident));

                    Self {
                        #(
                            #field_names: Vec::new(),
                        )*
                    }
                }


                /// Create a new layout struct with all internal vectors initialized.
                pub fn with_capacity(size: usize) -> Self {
                    // println!("Using struct-of-arrays for {}", stringify!(#struct_ident));

                    Self {
                        #(
                            #field_names: Vec::with_capacity(size),
                        )*
                    }
                }

                /// Get the number of elements in the layout
                pub fn len(&self) -> usize {
                    self.#first_field.len()
                }

                /// Returns `true` if the layout is empty
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }

                /// Add an instance of the original struct.
                /// Each field value is pushed into its corresponding vector.
                /// Returns the index of the newly inserted element.
                pub fn add(&mut self, item: #struct_ident #impl_generics) -> #id_ident {
                    let id = #id_ident(self.#first_field.len() as u32);

                    #(
                        self.#field_names.push(item.#field_names);
                    )*

                    id
                }

                #(
                    pub fn #field_names(&self) -> impl Iterator<Item = &#field_types> {
                        self.#field_names.iter()
                    }
                )*

                /// Returns a reference to the field value at the given index.
                // Generate an individual getter for each field.
                #(
                    /// Returns a reference to the field value at the given index.
                    pub fn #getter_names(&self, index: #id_ident) -> Result<&#field_types, #error_ident> {
                        self
                        .#field_names
                        .get(index.0 as usize)
                        .ok_or_else(|| #error_ident::#error_names)
                    }
                )*

                // Generate an individual getter for each field.
                #(
                    /// Returns a reference to the field value at the given index.
                    pub fn #getter_enumerated_names(&self) -> impl Iterator<Item = (#id_ident, &#field_types)>{
                        self
                        .#field_names
                        .iter()
                        .enumerate()
                        .map(|(index, item)| (#id_ident(index as u32), item))
                    }
                )*

                // Generate an mut individual getter for each field.
                #(
                    /// Returns a reference to the field value at the given index.
                    pub fn #getter_mut_names(&mut self, index: #id_ident) -> Result<&mut #field_types, #error_ident> {
                        self
                        .#field_names
                        .get_mut(index.0 as usize)
                        .ok_or_else(|| #error_ident::#error_names)
                    }
                )*
            }

        };

        output.into()
    } else if layout == Layout::ArrayOfStructs {
        let output = quote! {
            #both

            /// Layout version using array-of-structs layout.
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #layout_struct_ident #impl_generics #where_clause {
                pub data: Vec<#struct_ident #impl_generics>,
            }

            impl #impl_generics #layout_struct_ident #impl_generics #where_clause {
                /// Create a new layout struct with an empty data vector.
                pub fn new() -> Self {
                    // println!("Using array-of-structs for {}", stringify!(#struct_ident));

                    Self {
                        data: Vec::new(),
                    }
                }

                /// Create a new layout struct with a pre-allocated vector
                pub fn with_capacity(size: usize) -> Self {
                    // println!("Using array-of-structs for {}", stringify!(#struct_ident));

                    Self {
                        data: Vec::with_capacity(size),
                    }
                }

                pub fn len(&self) -> usize {
                    self.data.len()
                }

                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }

                /// Add an instance of the original struct.
                /// The entire struct is pushed into the internal vector.
                /// Returns the index of the newly inserted element.
                pub fn add(&mut self, item: #struct_ident #impl_generics) -> #id_ident {
                    let id = #id_ident(self.data.len() as u32);
                    self.data.push(item);
                    id
                }

                #(
                    pub fn #field_names(&self) -> impl Iterator<Item = &#field_types> {
                        self.data.iter().map(|item| &item.#field_names)
                    }
                )*

                // Generate an individual getter for each field.
                #(
                    /// Returns a reference to the field value at the given index.
                    pub fn #getter_names(&self, index: #id_ident) -> Result<&#field_types, #error_ident> {
                        self
                        .data
                        .get(index.0 as usize)
                        .map(|item| &item.#field_names)
                        .ok_or_else(|| #error_ident::#error_names)
                    }
                )*

                // Generate an individual getter for each field.
                #(
                    /// Returns a reference to the field value at the given index.
                    pub fn #getter_enumerated_names(&self) -> impl Iterator<Item = (#id_ident, &#field_types)>{
                        self
                        .data
                        .iter()
                        .enumerate()
                        .map(|(index, item)| (#id_ident(index as u32), &item.#field_names))
                    }
                )*

                // Generate an individual mut getter for each field.
                #(
                    /// Returns a reference to the field value at the given index.
                    pub fn #getter_mut_names(&mut self, index: #id_ident) -> Result<&mut #field_types, #error_ident> {
                        self
                        .data
                        .get_mut(index.0 as usize)
                        .map(|item| &mut item.#field_names)
                        .ok_or_else(|| #error_ident::#error_names)
                    }
                )*
            }
        };
        output.into()
    } else {
        syn::Error::new_spanned(
            struct_ident,
            "Invalid layout specified. Expected \"soa\" or \"aos\".",
        )
        .to_compile_error()
        .into()
    }
}

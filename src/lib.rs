use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Layout {
    StructOfArrays,
    ArrayOfStructs,
}

/// You can optionally select the underlying storage layout:
/// - Default (or `kind = "soa"`): struct-of-arrays (each field in its own Vec).
/// - `kind = "aos"`: array-of-structs (a single Vec of the original struct).
///
/// In both cases the generated type (named `<OriginalName>Layout`) has:
///   - `new() -> Self`
///   - `add(item: OriginalStruct) -> usize`
///   - For each field, a getter method (e.g. `get_fieldname(&self, index: usize) -> Option<&FieldType>`)

/// Implement a Struct-of-Arrays or Array-of-Structs collection of a single struct
///
/// Example:
///
/// ```rust
/// use core::error::Error;
/// /// The struct `NodesLayout` is created as a `struct-of-arrays`
/// #[soaaos::layout("struct-of-arrays")]
/// // #[layout("aos")] // For Array-of-Structs
/// struct Node {
///   name: String,
///   operation: u8,
///   arg1: u16,
///   arg2: u16,
/// }
///
/// ```
#[proc_macro_attribute]
pub fn layout(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input item as a DeriveInput (i.e. a struct definition).
    let input = parse_macro_input!(item as DeriveInput);

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
    let layout_struct_ident =
        syn::Ident::new(&format!("{}sLayout", struct_ident), struct_ident.span());
    let _layout_struct_ident_soa =
        syn::Ident::new(&format!("{}sLayoutSoa", struct_ident), struct_ident.span());
    let _layout_struct_ident_aos =
        syn::Ident::new(&format!("{}sLayoutAos", struct_ident), struct_ident.span());
    let error_ident = syn::Ident::new(&format!("{}sError", struct_ident), struct_ident.span());
    let id_ident = syn::Ident::new(&format!("{}Id", struct_ident), struct_ident.span());

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
    let getter_names: Vec<syn::Ident> = field_names
        .iter()
        .map(|ident| syn::Ident::new(&format!("get_{}", ident), ident.span()))
        .collect();

    // Create getter method names for each field enumerated (e.g. get_field1_enumerated).
    let getter_enumerated_names: Vec<syn::Ident> = field_names
        .iter()
        .map(|ident| syn::Ident::new(&format!("get_{}_enumerated", ident), ident.span()))
        .collect();

    // Create getter mut method names for each field (e.g. get_field1_mut).
    let getter_mut_names: Vec<syn::Ident> = field_names
        .iter()
        .map(|ident| syn::Ident::new(&format!("get_{}_mut", ident), ident.span()))
        .collect();

    // Create getter method names for each field (e.g. get_field1).
    let error_names: Vec<syn::Ident> = field_names
        .iter()
        .map(|ident| syn::Ident::new(&format!("NotFound_{}", ident), ident.span()))
        .collect();

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

        impl #layout_struct_ident {
            pub fn diff(&self, other: &#layout_struct_ident) -> Option<String> {
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
        }
    };

    // Generate different implementations based on the chosen layout.
    if layout == Layout::StructOfArrays {
        let output = quote! {
            #both

            /// Layout version using struct-of-arrays layout.
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #layout_struct_ident {
                #(
                    pub #field_names: Vec<#field_types>,
                )*
            }

            impl #layout_struct_ident {
                /// Create a new layout struct with all internal vectors initialized.
                pub fn new() -> Self {
                    println!("Using struct-of-arrays");

                    Self {
                        #(
                            #field_names: Vec::new(),
                        )*
                    }
                }

                pub fn len(&self) -> usize {
                    self.#first_field.len()
                }

                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }

                /// Add an instance of the original struct.
                /// Each field value is pushed into its corresponding vector.
                /// Returns the index of the newly inserted element.
                pub fn add(&mut self, item: #struct_ident) -> #id_ident {
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
            pub struct #layout_struct_ident {
                pub data: Vec<#struct_ident>,
            }

            impl #layout_struct_ident {
                /// Create a new layout struct with an empty data vector.
                pub fn new() -> Self {
                    println!("Using array-of-structs");

                    Self {
                        data: Vec::new(),
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
                pub fn add(&mut self, item: #struct_ident) -> #id_ident {
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

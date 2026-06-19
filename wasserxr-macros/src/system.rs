// `format_ident!` builds Rust identifiers such as `wxr_system_my_system`.
// `quote!` lets us write Rust-looking output code and turn it into tokens.
use quote::{format_ident, quote};

// These `syn` types parse raw tokens into structured Rust syntax.
// That is easier than manually inspecting every punctuation token ourselves.
use syn::{
    // `Error` produces compile errors at the user's macro call site.
    Error,
    // `Ident` parses names like `entities`.
    Ident,
    // `ItemFn` parses the function that has `#[System(...)]` on it.
    ItemFn,
    // `LitStr` parses component names like `"Transform"`.
    LitStr,
    // `Result` is `syn::Result`, the normal result type for parsers.
    Result,
    // `Token` lets us ask for exact punctuation such as `=`.
    Token,
    // `Parse` is the trait for custom parsers.
    // `ParseStream` is the cursor-like input type those parsers read from.
    parse::{Parse, ParseStream},
};

// This is the structured form of the attribute arguments.
// For `#[System(entities = [["Transform"], ["Camera"]])]`, it stores:
// vec![vec!["Transform"], vec!["Camera"]].
pub(crate) struct Args {
    // The outer `Vec` is the system groups.
    // The inner `Vec` is the list of required components for one group.
    // `LitStr` keeps the original string literal tokens so `quote!` can reuse them later.
    entities: Vec<Vec<LitStr>>,
}

// Implementing `Parse` teaches `syn` how to parse `Args`
// from the tokens inside `#[System(...)]`.
impl Parse for Args {
    // `parse` consumes tokens from `input` and either returns `Args`
    // or a compile error.
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // Parse the first token as an identifier.
        // In the supported syntax, this should be `entities`.
        let key: Ident = input.parse()?;

        // Reject unknown keys early so typos like `entity = ...` become clear compiler errors.
        if key != "entities" {
            // `new_spanned` points the error at the wrong key in the user's attribute.
            return Err(Error::new_spanned(key, "expected `entities`"));
        }

        // Parse the exact `=` token after `entities`.
        // This enforces `entities = ...` rather than accepting looser syntax.
        input.parse::<Token![=]>()?;

        // Declare a parse stream that will represent the tokens inside the outer brackets.
        // For `entities = [["A"], ["B"]]`, this will hold `["A"], ["B"]`.
        let entities;

        // Parse the outer `[ ... ]` and bind its contents to `entities`.
        syn::bracketed!(entities in input);

        // Collect the parsed groups here.
        let mut groups = Vec::new();

        // Keep parsing groups until the outer bracket content is empty.
        while !entities.is_empty() {
            // Declare a parse stream for one inner group.
            // For `["Transform", "Mesh"]`, this will hold `"Transform", "Mesh"`.
            let group;

            // Parse one inner `[ ... ]` group from the outer `entities` stream.
            syn::bracketed!(group in entities);

            // Collect this group's component string literals here.
            let mut components = Vec::new();

            // Keep parsing string literals until this group's brackets are empty.
            while !group.is_empty() {
                // Parse one component literal such as `"Transform"` and store it.
                components.push(group.parse()?);

                // If the group is now empty, there is no comma left to consume.
                if group.is_empty() {
                    break;
                }

                // Parse the comma between component literals.
                // This accepts `"Transform", "Mesh"` and rejects missing separators.
                group.parse::<Token![,]>()?;
            }

            // Store the completed group in the outer group list.
            groups.push(components);

            // If the outer list is now empty, there is no comma left to consume.
            if entities.is_empty() {
                break;
            }

            // Parse the comma between groups.
            // This accepts `["A"], ["B"]` and rejects missing separators.
            entities.parse::<Token![,]>()?;
        }

        // After the outer entity list, allow either no more tokens or one trailing comma.
        // This means both `#[System(entities = [[...]])]` and
        // `#[System(entities = [[...]],)]` are accepted.
        if !input.is_empty() {
            // Parse that optional trailing comma.
            input.parse::<Token![,]>()?;

            // If anything remains after the comma, it is an unsupported attribute argument.
            if !input.is_empty() {
                return Err(input.error("unexpected tokens after `entities`"));
            }
        }

        // A system with no groups cannot be scheduled meaningfully by the current engine.
        if groups.is_empty() {
            // Point the error at the outer `entities = [...]` bracket content.
            return Err(Error::new(
                entities.span(),
                "`entities` must contain at least one group",
            ));
        }

        // Return the parsed attribute in the structured form used by code generation.
        Ok(Self { entities: groups })
    }
}

// This function contains the code-generation logic.
// Keeping it separate from the proc-macro entrypoint makes it easier to read and test mentally.
pub(crate) fn expand(args: Args, item: ItemFn) -> Result<proc_macro2::TokenStream> {
    // Read the user's Rust function name, for example `my_system`.
    let function_name = item.sig.ident.clone();

    // Convert the function identifier into a `String`.
    // This string becomes the WasserXR system id and symbol suffix.
    let system_id = function_name.to_string();

    // Compute the exported group static symbol.
    // `my_system` becomes `WXR_GROUPS_MY_SYSTEM`, matching `System::new`.
    let groups_name = format!("WXR_GROUPS_{}", system_id.to_uppercase());

    // Compute the exported selector function symbol.
    // `my_system` becomes `wxr_select_my_system`.
    let selector_name = format!("wxr_select_{}", system_id);

    // Compute the exported runner function symbol.
    // `my_system` becomes `wxr_system_my_system`.
    let runner_name = format!("wxr_system_{}", system_id);

    // Build a Rust identifier for the generated group static.
    // The identifier is the Rust item name; `export_name` below is the ABI symbol name.
    let groups_ident = format_ident!("{}", groups_name);

    // Build a Rust identifier for the generated selector function.
    let selector_ident = format_ident!("{}", selector_name);

    // Build a Rust identifier for the generated runner function.
    let runner_ident = format_ident!("{}", runner_name);

    // Infer the number of groups from the number of inner arrays in `entities`.
    // Example: `[["A"], ["B"]]` has group count 2.
    let group_count = args.entities.len();

    // Convert each parsed group into quoted Rust code.
    // If a group was parsed as `"Transform", "Mesh"`, this produces `&["Transform", "Mesh"]`.
    // Later, `#(#component_groups),*` repeats these quoted groups with commas between them.
    let component_groups = args.entities.iter().map(|components| {
        // `quote!` returns tokens, not a runtime value.
        // `#(#components),*` repeats the `LitStr` values as string literals.
        quote! {
            &[#(#components),*]
        }
    });

    // Return the final generated Rust code.
    // Everything inside `quote!` is code the user's crate will compile after expansion.
    Ok(quote! {
        // Re-emit the user's original function so normal Rust code can call it too.
        #item

        // Export the group count under the symbol name WasserXR searches for with `dlsym`.
        #[unsafe(export_name = #groups_name)]
        // This generated Rust item name is unique because it includes the function name.
        static #groups_ident: usize = #group_count;

        // Export the selector under `wxr_select_<system_id>`.
        #[unsafe(export_name = #selector_name)]
        // The selector receives an immutable scene pointer and an entity UUID byte pointer.
        unsafe extern "C" fn #selector_ident(
            // `Scene::run_system` passes `self as *const Scene`.
            scene: *const ::wasserxr::scene::Scene,
            // `Scene::run_system` passes `Uuid::as_bytes().as_ptr()` as `*const u8`.
            entity: *const u8,
        ) -> i32 {
            // Convert the raw scene pointer into a shared reference.
            // The function is unsafe because the caller must guarantee the pointer is valid.
            let scene = unsafe { &*scene };

            // View the 16 bytes behind the entity pointer as a byte slice.
            // UUIDs are exactly 16 bytes, so the generated code uses that fixed length.
            let bytes = unsafe { std::slice::from_raw_parts(entity, 16) };

            // Copy the 16 bytes into an owned `Uuid` value.
            // The selector needs a `Uuid` because `Scene::has_component` accepts entity ids as `Uuid`.
            let entity_id = ::wasserxr::Uuid::from_slice(bytes)
                .expect("WasserXR entity pointers must point to 16 UUID bytes");

            // Build a runtime view of the compile-time component groups.
            // `component_groups` was computed above from the macro input. For example,
            // `entities = [["Transform", "Mesh"], ["Camera"]]` expands here to:
            // `&[&["Transform", "Mesh"], &["Camera"]]`.
            let selectors: &[&[&str]] = &[
                #(#component_groups),*
            ];

            // Check every group in order so the first matching group wins.
            for (group, components) in selectors.iter().enumerate() {
                // A group matches only if the entity has every component listed for that group.
                if components
                    .iter()
                    .all(|component| scene.has_component(entity_id, component))
                {
                    // Return the matching group index as the C ABI expects.
                    return group as i32;
                }
            }

            // `-1` means the entity does not belong to any group for this system.
            -1
        }

        // Export the runner under `wxr_system_<system_id>`.
        #[unsafe(export_name = #runner_name)]
        // The runner receives raw ABI arrays from `Scene::run_system`.
        unsafe extern "C" fn #runner_ident(
            // Mutable scene pointer so the Rust system can mutate the scene.
            scene: *mut ::wasserxr::scene::Scene,
            // Pointer to one entity-pointer array per group.
            entities: *const *const *const u8,
            // Pointer to the group-size array.
            groups: *const usize,
        ) {
            // Copy the group sizes into a Rust `Vec<usize>`.
            // The length is the generated group static because the macro knows the group count.
            let groups = unsafe { std::slice::from_raw_parts(groups, #groups_ident) }.to_vec();

            // View the outer entity group pointer as a slice with one entry per group.
            let raw_groups = unsafe { std::slice::from_raw_parts(entities, #groups_ident) };

            // Allocate the friendly Rust entity structure passed to the user's function.
            let mut rust_entities: Vec<Vec<::wasserxr::Uuid>> =
                Vec::with_capacity(#groups_ident);

            // Iterate over every raw group and its index.
            // The index lets us read the corresponding group size from `groups`.
            for (group_index, group_entities) in raw_groups.iter().enumerate() {
                // Convert this group's raw entity-pointer array into a slice.
                // Its length is `groups[group_index]`, computed by `Scene::run_system`.
                let raw_entities =
                    unsafe { std::slice::from_raw_parts(*group_entities, groups[group_index]) };

                // Allocate the Rust UUID vector for this one group.
                let mut rust_group = Vec::with_capacity(groups[group_index]);

                // Convert each raw entity pointer in this group into a `Uuid`.
                for entity in raw_entities {
                    // Each entity pointer points to 16 UUID bytes.
                    let bytes = unsafe { std::slice::from_raw_parts(*entity, 16) };

                    // Copy the UUID bytes into the Rust group vector.
                    rust_group.push(
                        ::wasserxr::Uuid::from_slice(bytes)
                            .expect("WasserXR entity pointers must point to 16 UUID bytes"),
                    );
                }

                // Add the completed group to the outer entity vector.
                rust_entities.push(rust_group);
            }

            // Convert the raw mutable scene pointer into the `&mut Scene`
            // expected by the user's original Rust system function.
            let scene = unsafe { &mut *scene };

            // Call the user's original function with friendly Rust arguments.
            #function_name(scene, rust_entities, groups);
        }
    })
}

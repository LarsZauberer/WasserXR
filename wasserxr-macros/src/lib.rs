use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Ident, ItemFn, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct SystemArgs {
    entities: Vec<Vec<LitStr>>,
}

impl Parse for SystemArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let key: Ident = input.parse()?;
        if key != "entities" {
            return Err(Error::new_spanned(key, "expected `entities`"));
        }

        input.parse::<Token![=]>()?;

        let entities;
        syn::bracketed!(entities in input);

        let mut groups = Vec::new();
        while !entities.is_empty() {
            let group;
            syn::bracketed!(group in entities);

            let mut components = Vec::new();
            while !group.is_empty() {
                components.push(group.parse()?);
                if group.is_empty() {
                    break;
                }
                group.parse::<Token![,]>()?;
            }

            groups.push(components);

            if entities.is_empty() {
                break;
            }
            entities.parse::<Token![,]>()?;
        }

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                return Err(input.error("unexpected tokens after `entities`"));
            }
        }

        if groups.is_empty() {
            return Err(Error::new(
                entities.span(),
                "`entities` must contain at least one group",
            ));
        }

        Ok(Self { entities: groups })
    }
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn System(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SystemArgs);
    let item = parse_macro_input!(item as ItemFn);

    expand_system(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_system(args: SystemArgs, item: ItemFn) -> Result<proc_macro2::TokenStream> {
    let function_name = item.sig.ident.clone();
    let system_id = function_name.to_string();
    let groups_name = format!("WXR_GROUPS_{}", system_id.to_uppercase());
    let selector_name = format!("wxr_select_{}", system_id);
    let runner_name = format!("wxr_system_{}", system_id);

    let groups_ident = format_ident!("{}", groups_name);
    let selector_ident = format_ident!("{}", selector_name);
    let runner_ident = format_ident!("{}", runner_name);

    let group_count = args.entities.len();
    let component_groups = args.entities.iter().map(|components| {
        quote! {
            &[#(#components),*]
        }
    });

    Ok(quote! {
        #item

        #[unsafe(export_name = #groups_name)]
        static #groups_ident: usize = #group_count;

        #[unsafe(export_name = #selector_name)]
        unsafe extern "C" fn #selector_ident(
            scene: *const ::wasserxr::scene::Scene,
            entity: *const u8,
        ) -> i32 {
            let scene = unsafe { &*scene };
            let bytes = unsafe { std::slice::from_raw_parts(entity, 16) };
            let entity_id = ::wasserxr::Uuid::from_slice(bytes)
                .expect("WasserXR entity pointers must point to 16 UUID bytes");
            let selectors: &[&[&str]] = &[
                #(#component_groups),*
            ];

            for (group, components) in selectors.iter().enumerate() {
                if components
                    .iter()
                    .all(|component| scene.has_component(entity_id, component))
                {
                    return group as i32;
                }
            }

            -1
        }

        #[unsafe(export_name = #runner_name)]
        unsafe extern "C" fn #runner_ident(
            scene: *mut ::wasserxr::scene::Scene,
            entities: *const *const *const u8,
            groups: *const usize,
        ) {
            let groups = unsafe { std::slice::from_raw_parts(groups, #groups_ident) }.to_vec();
            let raw_groups = unsafe { std::slice::from_raw_parts(entities, #groups_ident) };
            let mut rust_entities: Vec<Vec<::wasserxr::Uuid>> =
                Vec::with_capacity(#groups_ident);

            for (group_index, group_entities) in raw_groups.iter().enumerate() {
                let raw_entities =
                    unsafe { std::slice::from_raw_parts(*group_entities, groups[group_index]) };
                let mut rust_group = Vec::with_capacity(groups[group_index]);

                for entity in raw_entities {
                    let bytes = unsafe { std::slice::from_raw_parts(*entity, 16) };
                    rust_group.push(
                        ::wasserxr::Uuid::from_slice(bytes)
                            .expect("WasserXR entity pointers must point to 16 UUID bytes"),
                    );
                }

                rust_entities.push(rust_group);
            }

            let scene = unsafe { &mut *scene };
            #function_name(scene, rust_entities, groups);
        }
    })
}

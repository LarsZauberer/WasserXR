use quote::{format_ident, quote};
use syn::{
    Error, Ident, ItemFn, LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct Args {
    entities: Vec<Vec<LitStr>>,
}

pub(crate) struct LifecycleArgs {
    system_id: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                entities: Vec::new(),
            });
        }

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

        Ok(Self { entities: groups })
    }
}

impl Parse for LifecycleArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let system_id = input.parse()?;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;

            if !input.is_empty() {
                return Err(input.error("unexpected tokens after system name"));
            }
        }

        Ok(Self { system_id })
    }
}

pub(crate) fn expand(args: Args, item: ItemFn) -> Result<proc_macro2::TokenStream> {
    let function_name = item.sig.ident.clone();
    let system_id = function_name.to_string();
    let symbols = SystemSymbols::new(&system_id);
    let group_count = args.entities.len();

    let groups = create_groups_static(&symbols, group_count);
    let selector = create_selector_function(&symbols, &args.entities);
    let runner = create_runner_function(&symbols, &function_name);

    Ok(quote! {
        #item

        #groups
        #selector
        #runner
    })
}

pub(crate) fn expand_attacher(
    args: LifecycleArgs,
    item: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    expand_lifecycle("wxr_attach", args.system_id, item)
}

pub(crate) fn expand_detacher(
    args: LifecycleArgs,
    item: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    expand_lifecycle("wxr_detach", args.system_id, item)
}

struct SystemSymbols {
    groups_name: String,
    selector_name: String,
    runner_name: String,
    groups_ident: Ident,
    selector_ident: Ident,
    runner_ident: Ident,
}

impl SystemSymbols {
    fn new(system_id: &str) -> Self {
        let groups_name = format!("WXR_GROUPS_{}", system_id.to_uppercase());
        let selector_name = format!("wxr_select_{}", system_id);
        let runner_name = format!("wxr_system_{}", system_id);
        let groups_ident = format_ident!("{}", groups_name);
        let selector_ident = format_ident!("{}", selector_name);
        let runner_ident = format_ident!("{}", runner_name);

        Self {
            groups_name,
            selector_name,
            runner_name,
            groups_ident,
            selector_ident,
            runner_ident,
        }
    }
}

fn create_groups_static(symbols: &SystemSymbols, group_count: usize) -> proc_macro2::TokenStream {
    let groups_name = &symbols.groups_name;
    let groups_ident = &symbols.groups_ident;

    quote! {
        #[doc = "Generated WasserXR system group-count binding."]
        #[doc = ""]
        #[doc = "Stores how many entity groups this system selector can return."]
        #[unsafe(export_name = #groups_name)]
        static #groups_ident: usize = #group_count;
    }
}

fn create_selector_function(
    symbols: &SystemSymbols,
    entities: &[Vec<LitStr>],
) -> proc_macro2::TokenStream {
    let selector_name = &symbols.selector_name;
    let selector_ident = &symbols.selector_ident;
    let component_groups = create_component_groups(entities);

    quote! {
        #[doc = "Generated WasserXR system selector binding."]
        #[doc = ""]
        #[doc = "Returns the first matching entity group index for an entity, or -1 when no group matches."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`scene` must point to a valid `Scene`."]
        #[unsafe(export_name = #selector_name)]
        unsafe extern "C" fn #selector_ident(
            scene: *const ::wasserxr::scene::Scene,
            entity: ::wasserxr::bindings::scene::WXREntity,
        ) -> i32 {
            let scene = unsafe { &*scene };
            let entity_id = ::wasserxr::Uuid::from_bytes(entity.bytes);
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
    }
}

fn create_runner_function(
    symbols: &SystemSymbols,
    function_name: &Ident,
) -> proc_macro2::TokenStream {
    let runner_name = &symbols.runner_name;
    let runner_ident = &symbols.runner_ident;
    let groups_ident = &symbols.groups_ident;

    quote! {
        #[doc = "Generated WasserXR system runner binding."]
        #[doc = ""]
        #[doc = "Converts raw grouped entity UUID pointers into Rust vectors and calls the annotated system function."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`scene` must point to a valid `Scene`. `entities` and `groups` must describe the group layout declared by the generated group-count binding."]
        #[unsafe(export_name = #runner_name)]
        unsafe extern "C" fn #runner_ident(
            scene: *mut ::wasserxr::scene::Scene,
            entities: *const *const ::wasserxr::bindings::scene::WXREntity,
            groups: *const usize,
        ) {
            let groups = unsafe { std::slice::from_raw_parts(groups, #groups_ident) };
            let raw_groups = unsafe { std::slice::from_raw_parts(entities, #groups_ident) };
            let mut rust_entities: Vec<Vec<::wasserxr::Uuid>> =
                Vec::with_capacity(#groups_ident);

            for (group_index, group_entities) in raw_groups.iter().enumerate() {
                let raw_entities =
                    unsafe { std::slice::from_raw_parts(*group_entities, groups[group_index]) };
                let mut rust_group = Vec::with_capacity(groups[group_index]);

                for entity in raw_entities {
                    rust_group.push(::wasserxr::Uuid::from_bytes(entity.bytes));
                }

                rust_entities.push(rust_group);
            }

            let scene = unsafe { &mut *scene };
            #function_name(scene, rust_entities);
        }
    }
}

fn create_component_groups(entities: &[Vec<LitStr>]) -> Vec<proc_macro2::TokenStream> {
    entities
        .iter()
        .map(|components| {
            quote! {
                &[#(#components),*]
            }
        })
        .collect()
}

fn expand_lifecycle(
    symbol_prefix: &str,
    system_id: Ident,
    item: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    let function_name = item.sig.ident.clone();
    let wrapper_name = format!("{}_{}", symbol_prefix, system_id);
    let wrapper_ident = format_ident!("{}", wrapper_name);

    Ok(quote! {
        #item

        #[doc = "Generated WasserXR system lifecycle binding."]
        #[doc = ""]
        #[doc = "Calls the annotated attach or detach lifecycle function."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`scene` must point to a valid `Scene`."]
        #[unsafe(export_name = #wrapper_name)]
        unsafe extern "C" fn #wrapper_ident(scene: *mut ::wasserxr::scene::Scene) {
            let scene = unsafe { &mut *scene };
            #function_name(scene);
        }
    })
}

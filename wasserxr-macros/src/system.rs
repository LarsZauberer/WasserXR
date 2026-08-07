use quote::{format_ident, quote};
use syn::{
    Ident, ItemFn, Result, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct Args;

pub(crate) struct LifecycleArgs {
    system_id: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            Ok(Self)
        } else {
            Err(input.error(
                "`system` no longer accepts entity groups; declare them in the plugin manifest",
            ))
        }
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

pub(crate) fn expand(_args: Args, item: ItemFn) -> Result<proc_macro2::TokenStream> {
    let function_name = item.sig.ident.clone();
    let runner_ident = format_ident!("wxr_system_{}", function_name);

    Ok(quote! {
        #item

        #[doc = "Generated WasserXR system runner callback."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = "The scene and entity-group pointer/count collections must satisfy the system runner ABI contract."]
        pub unsafe extern "C" fn #runner_ident(
            scene: *mut ::wasserxr::scene::Scene,
            delta: f32,
            entities: *const *const ::wasserxr::bindings::scene::WXREntity,
            entity_counts: *const usize,
            entity_group_count: usize,
        ) {
            let counts: &[usize] = if entity_group_count == 0 {
                &[]
            } else {
                unsafe { ::std::slice::from_raw_parts(entity_counts, entity_group_count) }
            };
            let raw_groups: &[*const ::wasserxr::bindings::scene::WXREntity] =
                if entity_group_count == 0 {
                    &[]
                } else {
                    unsafe { ::std::slice::from_raw_parts(entities, entity_group_count) }
                };
            let mut rust_entities = ::std::vec::Vec::with_capacity(entity_group_count);
            for (group, count) in raw_groups.iter().zip(counts) {
                let raw_entities = if *count == 0 {
                    &[][..]
                } else {
                    unsafe { ::std::slice::from_raw_parts(*group, *count) }
                };
                rust_entities.push(
                    raw_entities
                        .iter()
                        .map(|entity| ::wasserxr::Uuid::from_bytes(entity.bytes))
                        .collect(),
                );
            }
            #function_name(unsafe { &mut *scene }, delta, rust_entities);
        }
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

fn expand_lifecycle(
    prefix: &str,
    system_id: Ident,
    item: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    let function_name = item.sig.ident.clone();
    let wrapper = format_ident!("{}_{}", prefix, system_id);
    Ok(quote! {
        #item

        #[doc = "Generated WasserXR system lifecycle callback."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = "`scene` must point to a valid `Scene`."]
        pub unsafe extern "C" fn #wrapper(scene: *mut ::wasserxr::scene::Scene) {
            #function_name(unsafe { &mut *scene });
        }
    })
}

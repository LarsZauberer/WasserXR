use crate::{
    definitions::{
        Definition,
        error::{AssetFieldDefinitionError, ComponentFieldDefinitionError},
        fields::{
            AssetFieldDefinition, ComponentFieldDefinition, Deserializer, Getter, Serializer,
        },
    },
    manifests::Manifest,
};

#[derive(Debug, Clone)]
pub struct ComponentFieldManifest {
    pub name: String,

    pub mutable: bool,
    pub getter: Option<Getter>,
    pub serializer: Option<Serializer>,
    pub deserializer: Option<Deserializer>,
}

impl Manifest<ComponentFieldDefinition> for ComponentFieldManifest {
    unsafe fn checked_convert(
        value: ComponentFieldDefinition,
    ) -> Result<Self, ComponentFieldDefinitionError> {
        unsafe { value.validate()? };
        Ok(Self {
            name: unsafe { value.name() }.expect("validated definitions have valid names"),
            mutable: value.mutable != 0,
            getter: value.getter,
            serializer: value.serializer,
            deserializer: value.deserializer,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AssetFieldManifest {
    pub name: String,
    pub getter: Getter,
}

impl Manifest<AssetFieldDefinition> for AssetFieldManifest {
    unsafe fn checked_convert(
        value: AssetFieldDefinition,
    ) -> Result<Self, AssetFieldDefinitionError> {
        unsafe { value.validate()? };
        Ok(Self {
            name: unsafe { value.name() }.expect("validated definitions have valid names"),
            getter: value
                .getter
                .expect("validated asset field definitions have a getter"),
        })
    }
}

#[cfg(test)]
mod component_field_tests {
    use std::ffi::c_void;

    use rstest::{fixture, rstest};

    use super::*;

    unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[fixture]
    fn component_field() -> ComponentFieldDefinition {
        static NAME: &[u8] = b"position\0";

        ComponentFieldDefinition {
            name: NAME.as_ptr().cast(),
            getter: Some(getter),
            mutable: 1,
            serializer: None,
            deserializer: None,
        }
    }

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn converts_component_field(
        mut component_field: ComponentFieldDefinition,
        #[case] mutable: bool,
    ) {
        component_field.mutable = i32::from(mutable);

        let manifest = unsafe { ComponentFieldManifest::checked_convert(component_field) }.unwrap();

        assert_eq!(manifest.name, "position");
        assert_eq!(manifest.mutable, mutable);
        assert!(manifest.getter.is_some());
        assert!(manifest.serializer.is_none());
        assert!(manifest.deserializer.is_none());
    }

    #[rstest]
    fn rejects_invalid_component_field(mut component_field: ComponentFieldDefinition) {
        component_field.mutable = 1;
        component_field.getter = None;

        assert_eq!(
            unsafe { ComponentFieldManifest::checked_convert(component_field).unwrap_err() },
            ComponentFieldDefinitionError::MutableButNoGetter("position".to_owned())
        );
    }
}

#[cfg(test)]
mod asset_field_tests {
    use std::ffi::c_void;

    use rstest::{fixture, rstest};

    use super::*;

    unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[fixture]
    fn asset_field() -> AssetFieldDefinition {
        static NAME: &[u8] = b"material\0";

        AssetFieldDefinition {
            name: NAME.as_ptr().cast(),
            getter: Some(getter),
        }
    }

    #[rstest]
    fn converts_asset_field(asset_field: AssetFieldDefinition) {
        let manifest = unsafe { AssetFieldManifest::checked_convert(asset_field) }.unwrap();

        assert_eq!(manifest.name, "material");
        assert!(std::ptr::fn_addr_eq(manifest.getter, getter as Getter));
    }
}

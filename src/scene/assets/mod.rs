//! Asset schema support used by asset plugins and scene asset queries.

/// Asset field metadata.
pub mod field;
/// Asset schema storage.
pub mod schema;

use std::ffi::{CString, c_char, c_void};

use crate::{
    error::AssetError,
    scene::{Scene, plugin::Plugin},
};

pub use field::{Field, Getter};
pub use schema::Schema;

pub(crate) type Creator = unsafe extern "C" fn(*mut Scene, *const c_char) -> *mut c_void;
pub(crate) type Destroyer = unsafe extern "C" fn(*mut Scene, *mut c_void);
pub(crate) type SchemaCreator = unsafe extern "C" fn(*mut Schema);

pub(crate) struct AssetType {
    id: String,
    plugin_id: String,
    creator: Creator,
    destroyer: Destroyer,
    schema: Schema,
}

impl AssetType {
    pub(crate) fn new(id: String, plugin: &Plugin, scene: &Scene) -> Result<Self, AssetError> {
        let plugin_id = plugin.get_id().to_owned();

        let creator_symbol = "wxr_asset_create_".to_owned() + &id;
        let schema_symbol = "wxr_asset_schema_".to_owned() + &id;
        let destroyer_symbol = "wxr_asset_destroy_".to_owned() + &id;

        let creator: Creator = plugin
            .get_symbol::<Creator>(&creator_symbol)
            .map_err(|error| {
                crate::debug!(scene, "Asset type `{}` has no creator function", id);
                AssetError::NoCreator(error)
            })?;

        let schema_creator: SchemaCreator = plugin
            .get_symbol::<SchemaCreator>(&schema_symbol)
            .map_err(|error| {
                crate::debug!(scene, "Asset type `{}` has no schema function", id);
                AssetError::NoSchema(error)
            })?;

        let destroyer: Destroyer = plugin.get_symbol(&destroyer_symbol).map_err(|error| {
            crate::debug!(scene, "Asset type `{}` has no destroyer function", id);
            AssetError::NoDestroyer(error)
        })?;

        let mut schema = Schema::default();
        unsafe {
            schema_creator(&mut schema as *mut Schema);
        }

        crate::info!(scene, "Asset type `{}` created", id);
        Ok(Self {
            id,
            plugin_id,
            creator,
            destroyer,
            schema,
        })
    }

    #[cfg(test)]
    pub(crate) fn create_asset(
        &self,
        scene: &mut Scene,
        data_string: &str,
    ) -> Result<Asset, AssetError> {
        Self::create_asset_with(scene, data_string, self.creator, self.destroyer)
    }

    pub(crate) fn create_asset_with(
        scene: &mut Scene,
        data_string: &str,
        creator: Creator,
        destroyer: Destroyer,
    ) -> Result<Asset, AssetError> {
        let data_string = CString::new(data_string).map_err(|_| AssetError::InvalidAsset)?;

        let data = unsafe { creator(scene as *mut Scene, data_string.as_ptr()) };

        if data.is_null() {
            return Err(AssetError::InvalidAsset);
        }

        Ok(Asset::new(data, destroyer))
    }

    pub(crate) fn creator(&self) -> Creator {
        self.creator
    }

    pub(crate) fn destroyer(&self) -> Destroyer {
        self.destroyer
    }

    pub(crate) unsafe fn get_field_ptr(
        &self,
        asset: &Asset,
        field: &str,
    ) -> Result<*mut c_void, AssetError> {
        let field_ptr = unsafe { self.schema.get_field_ptr(field, asset.data()) };

        field_ptr
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

pub(crate) struct Asset {
    data: *mut c_void,
    destroyer: Destroyer,
}

impl Asset {
    fn new(data: *mut c_void, destroyer: Destroyer) -> Self {
        Self { data, destroyer }
    }

    fn data(&self) -> *mut c_void {
        self.data
    }

    pub(crate) fn destroy(self, scene: &mut Scene) {
        unsafe {
            (self.destroyer)(scene as *mut Scene, self.data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::PluginError, scene::component::FieldType};
    use std::sync::{
        LazyLock, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    #[repr(C)]
    struct TestAsset {
        value: i64,
    }

    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
    static ASSET_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    unsafe extern "C" fn unit_asset_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut TestAsset)).value as *mut i64 as *mut c_void }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_create_unit_asset(
        _scene: *mut Scene,
        _data: *const c_char,
    ) -> *mut c_void {
        Box::into_raw(Box::new(TestAsset { value: 5 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_schema_unit_asset(schema: *mut Schema) {
        unsafe {
            (*schema).add_field("value".to_owned(), FieldType::I64, Some(unit_asset_getter));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_destroy_unit_asset(_scene: *mut Scene, data: *mut c_void) {
        DROP_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe {
            drop(Box::from_raw(data as *mut TestAsset));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_create_null_asset(
        _scene: *mut Scene,
        _data: *const c_char,
    ) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_schema_null_asset(_schema: *mut Schema) {}

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_destroy_null_asset(_scene: *mut Scene, _data: *mut c_void) {}

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_create_missing_schema(
        _scene: *mut Scene,
        _data: *const c_char,
    ) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_destroy_missing_schema(_scene: *mut Scene, _data: *mut c_void) {}

    #[test]
    fn asset_type_new_with_static_symbols() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();
        let asset_type = AssetType::new("unit_asset".to_owned(), &plugin, &scene).unwrap();

        assert_eq!(asset_type.get_id(), "unit_asset");
        assert_eq!(asset_type.get_plugin_id(), "");
    }

    #[test]
    fn asset_type_create_asset() {
        let _guard = ASSET_TEST_LOCK.lock().unwrap();
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let asset_type = AssetType::new("unit_asset".to_owned(), &plugin, &scene).unwrap();
        let asset = asset_type.create_asset(&mut scene, "path").unwrap();

        let field = unsafe { asset_type.get_field_ptr(&asset, "value").unwrap() };

        assert_eq!(unsafe { *(field as *const i64) }, 5);
        asset.destroy(&mut scene);
    }

    #[test]
    fn asset_type_create_asset_rejects_null_asset() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let asset_type = AssetType::new("null_asset".to_owned(), &plugin, &scene).unwrap();

        assert_eq!(
            asset_type.create_asset(&mut scene, "path").err(),
            Some(AssetError::InvalidAsset)
        );
    }

    #[test]
    fn asset_type_new_without_creator() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();

        let result = AssetType::new("missing_creator".to_owned(), &plugin, &scene);

        assert!(matches!(
            result,
            Err(AssetError::NoCreator(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_asset_create_missing_creator"
        ));
    }

    #[test]
    fn asset_type_new_without_schema() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();

        let result = AssetType::new("missing_schema".to_owned(), &plugin, &scene);

        assert!(matches!(
            result,
            Err(AssetError::NoSchema(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_asset_schema_missing_schema"
        ));
    }

    #[test]
    fn asset_drop_existing_asset() {
        let _guard = ASSET_TEST_LOCK.lock().unwrap();
        DROP_COUNT.store(0, Ordering::Relaxed);
        {
            let mut scene = Scene::new();
            let plugin = Plugin::new_static();
            let asset_type = AssetType::new("unit_asset".to_owned(), &plugin, &scene).unwrap();
            let asset = asset_type.create_asset(&mut scene, "path").unwrap();
            asset.destroy(&mut scene);
        }

        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
    }
}

use std::ffi::CString;

use wasserxr::scene::Scene;
use wasserxr_macros::asset_type_creator;

#[derive(Debug, PartialEq, Eq)]
struct TestAsset {
    source: String,
}

#[asset_type_creator(TestAsset)]
fn create_test_asset(_scene: &mut Scene, data: &str) -> Option<TestAsset> {
    Some(TestAsset {
        source: data.to_owned(),
    })
}

#[test]
fn asset_type_creator_returns_the_created_asset() {
    let mut scene = Scene::new();
    let data = CString::new("asset.dat").unwrap();

    let asset = unsafe { wxr_asset_create_TestAsset(&mut scene, data.as_ptr()) };
    assert!(!asset.is_null());

    let asset = unsafe { Box::from_raw(asset.cast::<TestAsset>()) };
    assert_eq!(asset.source, "asset.dat");
}

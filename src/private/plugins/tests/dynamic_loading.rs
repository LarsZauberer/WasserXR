use std::{cell::RefCell, path::Path, rc::Rc};

use mockall::{Sequence, mock};
use rstest::{fixture, rstest};

use crate::{
    definitions::plugins::PluginDefinition,
    errors::PluginError,
    private::{
        io::{FileIO, PluginIO},
        plugins::Plugin,
    },
    utils::version::Version,
};

mock! {
    IO {}
    impl FileIO for IO {
        type Error = PluginError;
        fn copy(src: &std::path::Path, dst: &std::path::Path) -> Result<(), PluginError>;
    }
    impl PluginIO for IO {
        type Error = PluginError;
        unsafe fn get_plugin_definition(src: &std::path::Path) -> Result<crate::definitions::plugins::PluginDefinition, PluginError>;
    }
}

#[fixture]
fn valid_plugin_definition() -> PluginDefinition {
    static NAME: &[u8] = b"valid\0";

    PluginDefinition {
        name: NAME.as_ptr().cast(),
        engine_version: Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        },
        components: std::ptr::null(),
        component_count: 0,
    }
}

#[rstest]
fn valid_dynamic_loading(valid_plugin_definition: PluginDefinition) {
    let path = Path::new("./valid.so");
    let copied_path = Rc::new(RefCell::new(None));

    let mut sequence = Sequence::new();

    let copy_context = MockIO::copy_context();
    let copied_path_for_copy = Rc::clone(&copied_path);
    copy_context
        .expect()
        .withf_st(move |src: &Path, dst: &Path| {
            let valid_call =
                src == Path::new("./valid.so") && dst.starts_with(std::env::temp_dir().as_path());
            if valid_call {
                copied_path_for_copy.replace(Some(dst.to_owned()));
            }
            valid_call
        })
        .once()
        .in_sequence(&mut sequence)
        .returning_st(|_, _| Ok(()));

    let plugin_context = MockIO::get_plugin_definition_context();
    let copied_path_for_definition = Rc::clone(&copied_path);
    plugin_context
        .expect()
        .withf_st(move |src: &Path| copied_path_for_definition.borrow().as_deref() == Some(src))
        .once()
        .in_sequence(&mut sequence)
        .returning_st(move |_| Ok(valid_plugin_definition));

    let plugin = unsafe { Plugin::load_shared::<MockIO>(path) };
    let plugin = plugin.expect("valid plugin should be loadable");

    assert_eq!(plugin.manifest.name, "valid");
}

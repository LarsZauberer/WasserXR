//! Synchronous component method calls over the plugin ABI.
//!
//! A [`Method`] resolves an exported `wxr_method_<component>_<name>` symbol in
//! the plugin that owns a component instance and calls it with named arguments.
//! The call is always synchronous and is never deferred by the scene.

use std::ffi::{CStr, c_char, c_void};
use std::marker::PhantomData;

use uuid::Uuid;

use crate::bindings::scene::WXRScene;
use crate::error::SceneError;
use crate::scene::Scene;

/// ABI-safe named argument passed to a component method.
///
/// Both the name and the data are borrowed from the caller and stay owned by
/// the caller. There is deliberately no type, size, or alignment metadata.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WXRArgument {
    /// Borrowed null-terminated argument name.
    pub name: *const c_char,
    /// Borrowed argument data pointer, cast unchecked by the callee.
    pub data: *mut c_void,
}

/// Status of a component method call or argument resolution.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WXRMethodStatus {
    /// The method ran and returned `Ok`.
    Success,
    /// A required argument name was not present.
    MissingArgument,
    /// A required argument name occurred more than once.
    DuplicateArgument,
    /// The method ran and returned `Err`.
    ActionError,
}

/// Result of a component method call.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WXRMethodResult {
    /// Overall outcome of the call.
    pub status: WXRMethodStatus,
    /// The `Err(code)` value when `status` is `ActionError`, otherwise zero.
    pub action_error: i32,
    /// The `Ok(value)` pointer when `status` is `Success`, otherwise null.
    pub value: *mut c_void,
}

/// Erased C-ABI signature shared by every generated method symbol.
pub type MethodFn = unsafe extern "C" fn(
    scene: *mut WXRScene,
    component: *mut c_void,
    arguments: *const WXRArgument,
    argument_count: usize,
) -> WXRMethodResult;

/// Resolves a single named argument by a linear scan over `arguments`.
///
/// The resolved data pointer is cast unchecked to `&mut T`. This is deliberately
/// unsound: there is no type, size, or alignment validation, and data pointers
/// are not checked for null. Returns [`WXRMethodStatus::MissingArgument`] when
/// `name` is absent and [`WXRMethodStatus::DuplicateArgument`] when it occurs
/// more than once. Extra unknown arguments are ignored.
pub fn find_argument<'a, T>(
    arguments: &[WXRArgument],
    name: &CStr,
) -> Result<&'a mut T, WXRMethodStatus> {
    let mut found: Option<*mut c_void> = None;

    for argument in arguments {
        if argument.name.is_null() {
            continue;
        }

        let argument_name = unsafe { CStr::from_ptr(argument.name) };
        if argument_name == name {
            if found.is_some() {
                return Err(WXRMethodStatus::DuplicateArgument);
            }
            found = Some(argument.data);
        }
    }

    match found {
        Some(pointer) => Ok(unsafe { &mut *(pointer as *mut T) }),
        None => Err(WXRMethodStatus::MissingArgument),
    }
}

/// A resolved, ready-to-call component method.
///
/// The Rust builder keeps a mutable scene borrow for the method's complete
/// lifetime, so safe Rust cannot remove the component, unload its plugin, or
/// otherwise mutate the scene between lookup and invocation. The call does not
/// re-resolve or revalidate the component or function pointer.
///
/// The type is also used directly as the opaque heap handle behind the C
/// `WXRMethod` binding, where the scene pointer is raw and the lifetime is
/// unbounded.
pub struct Method<'scene> {
    scene: *mut WXRScene,
    function: MethodFn,
    component: *mut c_void,
    arguments: Vec<WXRArgument>,
    _scene: PhantomData<&'scene mut Scene>,
}

impl<'scene> Method<'scene> {
    pub(crate) fn new(scene: *mut WXRScene, function: MethodFn, component: *mut c_void) -> Self {
        Self {
            scene,
            function,
            component,
            arguments: Vec::new(),
            _scene: PhantomData,
        }
    }

    pub(crate) fn push_argument(&mut self, name: *const c_char, data: *mut c_void) {
        self.arguments.push(WXRArgument { name, data });
    }

    /// Appends a borrowed named argument, erasing `value` to `*mut c_void`.
    pub fn argument<T>(mut self, name: &'scene CStr, value: &'scene mut T) -> Self {
        self.push_argument(name.as_ptr(), value as *mut T as *mut c_void);
        self
    }

    /// Invokes the method synchronously.
    pub fn call(&self) -> WXRMethodResult {
        unsafe {
            (self.function)(
                self.scene,
                self.component,
                self.arguments.as_ptr(),
                self.arguments.len(),
            )
        }
    }
}

impl Scene {
    pub(crate) fn resolve_method(
        &self,
        entity_id: Uuid,
        component_id: &str,
        name: &str,
    ) -> Result<(MethodFn, *mut c_void), SceneError> {
        let component = self.get_component(entity_id, component_id)?;
        let data = component.get_data();

        // Resolve the method only in the plugin that owns this component.
        let Some(plugin) = self.plugins.get(component.get_plugin_id()) else {
            return Err(SceneError::PluginNotFound);
        };

        let symbol = format!("wxr_method_{}_{}", component_id, name);
        let function = plugin
            .get_symbol::<MethodFn>(&symbol)
            .map_err(|_| SceneError::MethodNotFound)?;

        Ok((function, data))
    }

    /// Resolves a synchronous component method for a named argument call.
    ///
    /// The method is resolved from the exported `wxr_method_<component>_<name>`
    /// symbol in the plugin that owns the component instance. The returned
    /// [`Method`] holds a mutable scene borrow until it is consumed by
    /// [`Method::call`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut force = 9.0_f32;
    /// let result = scene
    ///     .get_method(entity, "Component", "my_method")?
    ///     .argument(c"force", &mut force)
    ///     .call();
    /// ```
    pub fn get_method(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        name: &str,
    ) -> Result<Method<'_>, SceneError> {
        let (function, component) = self.resolve_method(entity_id, component_id, name)?;
        Ok(Method::new(
            self as *mut Scene as *mut WXRScene,
            function,
            component,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argument(name: &CStr, data: *mut c_void) -> WXRArgument {
        WXRArgument {
            name: name.as_ptr(),
            data,
        }
    }

    #[test]
    fn find_argument_returns_single_match() {
        let mut value = 7_i32;
        let arguments = [argument(c"force", &mut value as *mut i32 as *mut c_void)];

        let found = find_argument::<i32>(&arguments, c"force").unwrap();
        assert_eq!(*found, 7);
    }

    #[test]
    fn find_argument_reports_missing() {
        let arguments: [WXRArgument; 0] = [];
        assert_eq!(
            find_argument::<i32>(&arguments, c"force").unwrap_err(),
            WXRMethodStatus::MissingArgument
        );
    }

    #[test]
    fn find_argument_reports_duplicate() {
        let mut value = 1_i32;
        let data = &mut value as *mut i32 as *mut c_void;
        let arguments = [argument(c"force", data), argument(c"force", data)];

        assert_eq!(
            find_argument::<i32>(&arguments, c"force").unwrap_err(),
            WXRMethodStatus::DuplicateArgument
        );
    }

    #[test]
    fn find_argument_ignores_unknown_and_null_names() {
        let mut value = 3_i32;
        let data = &mut value as *mut i32 as *mut c_void;
        let arguments = [
            argument(c"other", data),
            WXRArgument {
                name: std::ptr::null(),
                data,
            },
            argument(c"force", data),
        ];

        let found = find_argument::<i32>(&arguments, c"force").unwrap();
        assert_eq!(*found, 3);
    }
}

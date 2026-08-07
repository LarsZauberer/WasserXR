//! Synchronous component method calls over the plugin ABI.

use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, c_char, c_void},
    marker::PhantomData,
    rc::Rc,
};

use uuid::Uuid;

use crate::{
    bindings::scene::WXRScene,
    scene::{
        Scene, SceneError,
        component::{ComponentError, FieldType},
    },
};

pub use super::descriptor::{WXRComponentMethodDescriptor, WXRMethodArgumentDescriptor};

/// ABI-safe named argument supplied by a method caller.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WXRArgument {
    pub name: *const c_char,
    pub field_type: u32,
    pub data: *mut c_void,
}

/// Status of a component method call or argument resolution.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WXRMethodStatus {
    Success = 0,
    MissingArgument = 1,
    DuplicateArgument = 2,
    ActionError = 3,
    UnexpectedArgument = 4,
    TypeMismatch = 5,
    NullArgument = 6,
}

/// Result of a component method call.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WXRMethodResult {
    pub status: WXRMethodStatus,
    pub action_error: i32,
    pub value: *mut c_void,
}

/// Ordered C-ABI signature shared by every component method callback.
pub type MethodFn = unsafe extern "C" fn(
    scene: *mut WXRScene,
    component: *mut c_void,
    arguments: *const *mut c_void,
    argument_count: usize,
) -> WXRMethodResult;

pub(crate) struct MethodArgument {
    pub(crate) name: String,
    pub(crate) field_type: FieldType,
    pub(crate) nullable: bool,
}

pub(crate) struct MethodDefinition {
    callback: MethodFn,
    arguments: Vec<MethodArgument>,
}

impl MethodDefinition {
    pub(crate) fn new(callback: MethodFn, arguments: Vec<MethodArgument>) -> Self {
        Self {
            callback,
            arguments,
        }
    }

    #[cfg(test)]
    pub(crate) fn argument_is_nullable(&self, index: usize) -> bool {
        self.arguments[index].nullable
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Simple Rust values whose method argument type tag can be inferred safely.
pub trait MethodArgumentType: sealed::Sealed {
    const FIELD_TYPE: FieldType;
}

macro_rules! method_argument_types {
    ($($type:ty => $field_type:ident),* $(,)?) => {
        $(
            impl sealed::Sealed for $type {}
            impl MethodArgumentType for $type {
                const FIELD_TYPE: FieldType = FieldType::$field_type;
            }
        )*
    };
}

method_argument_types! {
    i8 => I8, i16 => I16, i32 => I32, i64 => I64, i128 => I128, isize => Isize,
    u8 => U8, u16 => U16, u32 => U32, u64 => U64, u128 => U128, usize => Usize,
    f32 => F32, f64 => F64, [f32; 2] => F32Vec2, [f32; 3] => F32Vec3,
    [f64; 2] => F64Vec2, [f64; 3] => F64Vec3, char => Char, String => String,
    bool => Boolean,
}

/// A resolved, ready-to-call component method.
pub struct Method<'scene> {
    scene: *mut WXRScene,
    definition: Rc<MethodDefinition>,
    component: *mut c_void,
    arguments: Vec<WXRArgument>,
    _scene: PhantomData<&'scene mut Scene>,
}

impl<'scene> Method<'scene> {
    pub(crate) fn new(
        scene: *mut WXRScene,
        definition: Rc<MethodDefinition>,
        component: *mut c_void,
    ) -> Self {
        Self {
            scene,
            definition,
            component,
            arguments: Vec::new(),
            _scene: PhantomData,
        }
    }

    pub(crate) fn push_argument(
        &mut self,
        name: *const c_char,
        field_type: u32,
        data: *mut c_void,
    ) {
        self.arguments.push(WXRArgument {
            name,
            field_type,
            data,
        });
    }

    /// Appends a non-null argument with an inferred simple type tag.
    pub fn argument<T: MethodArgumentType>(
        mut self,
        name: &'scene CStr,
        value: &'scene mut T,
    ) -> Self {
        self.push_argument(
            name.as_ptr(),
            T::FIELD_TYPE as u32,
            value as *mut T as *mut c_void,
        );
        self
    }

    /// Appends a nullable argument with an inferred simple type tag.
    pub fn nullable_argument<T: MethodArgumentType>(
        mut self,
        name: &'scene CStr,
        value: Option<&'scene mut T>,
    ) -> Self {
        let data = value.map_or(std::ptr::null_mut(), |value| value as *mut T as *mut c_void);
        self.push_argument(name.as_ptr(), T::FIELD_TYPE as u32, data);
        self
    }

    /// Appends an opaque non-null argument.
    ///
    /// # Safety
    /// The value layout must match the method callback's declared blob type.
    pub unsafe fn argument_blob<T>(mut self, name: &'scene CStr, value: &'scene mut T) -> Self {
        self.push_argument(
            name.as_ptr(),
            FieldType::Blob as u32,
            value as *mut T as *mut c_void,
        );
        self
    }

    /// Appends an opaque nullable argument.
    ///
    /// # Safety
    /// Any non-null value layout must match the method callback's declared blob type.
    pub unsafe fn nullable_argument_blob<T>(
        mut self,
        name: &'scene CStr,
        value: Option<&'scene mut T>,
    ) -> Self {
        let data = value.map_or(std::ptr::null_mut(), |value| value as *mut T as *mut c_void);
        self.push_argument(name.as_ptr(), FieldType::Blob as u32, data);
        self
    }

    /// Validates, orders, and invokes the method synchronously.
    pub fn call(&self) -> WXRMethodResult {
        let expected: HashMap<&str, (usize, &MethodArgument)> = self
            .definition
            .arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| (argument.name.as_str(), (index, argument)))
            .collect();
        let mut seen = HashSet::new();
        let mut ordered = vec![std::ptr::null_mut(); expected.len()];

        for supplied in &self.arguments {
            if supplied.name.is_null() {
                return method_error(WXRMethodStatus::UnexpectedArgument);
            }
            let Ok(name) = unsafe { CStr::from_ptr(supplied.name) }.to_str() else {
                return method_error(WXRMethodStatus::UnexpectedArgument);
            };
            if !seen.insert(name) {
                return method_error(WXRMethodStatus::DuplicateArgument);
            }
            let Some((index, declaration)) = expected.get(name).copied() else {
                return method_error(WXRMethodStatus::UnexpectedArgument);
            };
            if FieldType::try_from(supplied.field_type).ok() != Some(declaration.field_type) {
                return method_error(WXRMethodStatus::TypeMismatch);
            }
            if supplied.data.is_null() && !declaration.nullable {
                return method_error(WXRMethodStatus::NullArgument);
            }
            ordered[index] = supplied.data;
        }

        if seen.len() != expected.len() {
            return method_error(WXRMethodStatus::MissingArgument);
        }

        let arguments = if ordered.is_empty() {
            std::ptr::null()
        } else {
            ordered.as_ptr()
        };
        unsafe { (self.definition.callback)(self.scene, self.component, arguments, ordered.len()) }
    }
}

fn method_error(status: WXRMethodStatus) -> WXRMethodResult {
    WXRMethodResult {
        status,
        action_error: 0,
        value: std::ptr::null_mut(),
    }
}

impl Scene {
    pub(crate) fn resolve_method(
        &self,
        entity_id: Uuid,
        component_id: &str,
        name: &str,
    ) -> Result<(Rc<MethodDefinition>, *mut c_void), SceneError> {
        let component = self.get_component(entity_id, component_id)?;
        let definition = component
            .get_method(name)
            .ok_or(ComponentError::MethodNotFound)?;
        Ok((definition, component.get_data()))
    }

    /// Resolves a synchronous component method.
    pub fn get_method(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        name: &str,
    ) -> Result<Method<'_>, SceneError> {
        let (definition, component) = self.resolve_method(entity_id, component_id, name)?;
        Ok(Method::new(
            self as *mut Scene as *mut WXRScene,
            definition,
            component,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn callback(
        _scene: *mut WXRScene,
        _component: *mut c_void,
        arguments: *const *mut c_void,
        argument_count: usize,
    ) -> WXRMethodResult {
        assert_eq!(argument_count, 1);
        assert!(!arguments.is_null());
        WXRMethodResult {
            status: WXRMethodStatus::Success,
            action_error: 0,
            value: unsafe { *arguments },
        }
    }

    fn definition(nullable: bool) -> MethodDefinition {
        MethodDefinition::new(
            callback,
            vec![MethodArgument {
                name: "value".to_owned(),
                field_type: FieldType::I32,
                nullable,
            }],
        )
    }

    #[test]
    fn method_orders_and_calls_arguments() {
        let mut value = 7_i32;
        let mut method = Method::new(
            std::ptr::null_mut(),
            Rc::new(definition(false)),
            std::ptr::null_mut(),
        );
        method.push_argument(
            c"value".as_ptr(),
            FieldType::I32 as u32,
            (&mut value as *mut i32).cast(),
        );
        assert_eq!(method.call().value, (&mut value as *mut i32).cast());
    }

    #[test]
    fn method_rejects_type_mismatch() {
        let mut value = 7_u32;
        let mut method = Method::new(
            std::ptr::null_mut(),
            Rc::new(definition(false)),
            std::ptr::null_mut(),
        );
        method.push_argument(
            c"value".as_ptr(),
            FieldType::U32 as u32,
            (&mut value as *mut u32).cast(),
        );
        assert_eq!(method.call().status, WXRMethodStatus::TypeMismatch);
    }

    #[test]
    fn method_allows_declared_null() {
        let mut method = Method::new(
            std::ptr::null_mut(),
            Rc::new(definition(true)),
            std::ptr::null_mut(),
        );
        method.push_argument(
            c"value".as_ptr(),
            FieldType::I32 as u32,
            std::ptr::null_mut(),
        );
        assert_eq!(method.call().status, WXRMethodStatus::Success);
    }
}

//! The definition of components in WasserXR

use std::ffi::{CStr, CString, c_char, c_void};

use crate::definitions::{
    Definition, error::ComponentDefinitionError, fields::ComponentFieldDefinition,
};

/// Creator function for a component. It is a constructor for the component. It allocates the
/// component created onto the heap and then returns the raw pointer in form of a null pointer. The
/// creator function should not fail.
///
/// # Safety
///
/// There are no immediate preconditions for the function to be safe. The function might be some
/// function defined in some foreign language and is therefore inherently unsafe.
pub type Creator = unsafe extern "C" fn() -> *mut c_void;

/// Destroyer function for a component. It is basically a destructor call to the component. The
/// destroyer function should not fail.
/// It takes the pointer to the object created by the [`wasserxr::definitions::components::creator`]
/// and destroys the object.
///
/// # Safety
///
/// The function requires a pointer that was created by the corresponding creator function of the
/// component.
pub type Destroyer = unsafe extern "C" fn(ptr: *mut c_void);

/// This is the definition defining a component in WasserXR. It contains a pointer to all the
/// functions to create, destroy the actual components and what kind of fields are included.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ComponentDefinition {
    name: *const c_char,
    creator: Creator,
    destroyer: Destroyer,

    fields: &'static [ComponentFieldDefinition],
}

impl Definition for ComponentDefinition {
    type Error = ComponentDefinitionError;

    unsafe fn validate(&self) -> Result<(), Self::Error> {
        if self.name.is_null() {
            return Err(ComponentDefinitionError::NameIsNull);
        }

        let field_violation = self
            .fields
            .iter()
            .map(|field| unsafe { field.validate() })
            .find(|x| x.is_err());
        if let Some(Err(violation)) = field_violation {
            // TODO: Make the name is valid string check a bit earlier and make it it's own error
            let name = unsafe {
                CStr::from_ptr(self.name)
                    .to_owned()
                    .into_string()
                    .expect("")
            };
            return Err(ComponentDefinitionError::FieldInvalid(name, violation));
        }

        Ok(())
    }
}

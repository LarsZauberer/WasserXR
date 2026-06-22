//! Typed component-field queries for `Scene::query` and `Scene::query_mut`.
//!
//! The query type decides how each requested field is borrowed. For example,
//! this asks for a shared reference to `value`:
//!
//! ```ignore
//! scene.query::<(&i64,)>(entity, "counter", &["value"])?;
//! ```
//!
//! This asks for a mutable reference to `a` and a shared reference to `b`:
//!
//! ```ignore
//! scene.query_mut::<(&mut i64, &i64)>(entity, "pair", &["a", "b"])?;
//! ```
//!
//! `QueryItem` maps raw field pointers to shared references for `query`.
//! `QueryItemMut` maps raw field pointers to either shared or mutable references
//! for `query_mut`. Its `REQUIRES_MUTABLE` constant is `false` for `&T` and
//! `true` for `&mut T`.
//!
//! `SceneQueryMut::for_each_mutable_field` uses that constant to connect tuple
//! positions to runtime field names. For a query like
//! `(&mut i64, &i64)` with `["a", "b"]`, it calls the provided check function
//! only for `"a"`. `Scene::query_mut` provides a check that looks up the
//! component schema and returns `FieldNotMutable` if that field was not
//! registered as mutable.
//!
//! Only after field count, duplicate fields, and mutable permissions have been
//! validated does `Scene` fetch the raw pointers and ask the tuple query traits
//! to build the final references.
//!
//! The raw `*mut c_void` is the internal escape hatch that lets one query build
//! several references from one component. `Scene::query_mut` validates field
//! count, duplicate fields, and mutable permissions before this module creates
//! references.

use std::ffi::c_void;

use crate::error::SceneError;

pub trait QueryItem<'scene>: Sized {
    unsafe fn fetch(field: *mut c_void) -> Result<Self, SceneError>;
}

impl<'scene, T: 'scene> QueryItem<'scene> for &'scene T {
    unsafe fn fetch(field: *mut c_void) -> Result<Self, SceneError> {
        Ok(unsafe { &*(field as *const T) })
    }
}

pub trait QueryItemMut<'scene>: Sized {
    const REQUIRES_MUTABLE: bool;

    unsafe fn fetch(field: *mut c_void) -> Result<Self, SceneError>;
}

impl<'scene, T: 'scene> QueryItemMut<'scene> for &'scene T {
    const REQUIRES_MUTABLE: bool = false;

    unsafe fn fetch(field: *mut c_void) -> Result<Self, SceneError> {
        Ok(unsafe { &*(field as *const T) })
    }
}

impl<'scene, T: 'scene> QueryItemMut<'scene> for &'scene mut T {
    const REQUIRES_MUTABLE: bool = true;

    unsafe fn fetch(field: *mut c_void) -> Result<Self, SceneError> {
        Ok(unsafe { &mut *(field as *mut T) })
    }
}

pub trait SceneQuery<'scene>: Sized {
    const FIELD_COUNT: usize;

    unsafe fn fetch(fields: &[*mut c_void]) -> Result<Self, SceneError>;
}

pub trait SceneQueryMut<'scene>: Sized {
    const FIELD_COUNT: usize;

    fn for_each_mutable_field<Check>(fields: &[&str], check: Check) -> Result<(), SceneError>
    where
        Check: FnMut(&str) -> Result<(), SceneError>;

    unsafe fn fetch(fields: &[*mut c_void]) -> Result<Self, SceneError>;
}

macro_rules! impl_scene_query_tuple {
    ($count:expr; $($name:ident: $index:tt),+) => {
        impl<'scene, $($name),+> SceneQuery<'scene> for ($($name,)+)
        where
            $($name: QueryItem<'scene>),+
        {
            const FIELD_COUNT: usize = $count;

            unsafe fn fetch(
                fields: &[*mut c_void],
            ) -> Result<Self, SceneError> {
                Ok(($(
                    unsafe { $name::fetch(fields[$index])? },
                )+))
            }
        }
    };
}

macro_rules! impl_scene_query_mut_tuple {
    ($count:expr; $($name:ident: $index:tt),+) => {
        impl<'scene, $($name),+> SceneQueryMut<'scene> for ($($name,)+)
        where
            $($name: QueryItemMut<'scene>),+
        {
            const FIELD_COUNT: usize = $count;

            fn for_each_mutable_field<Check>(
                fields: &[&str],
                mut check: Check,
            ) -> Result<(), SceneError>
            where
                Check: FnMut(&str) -> Result<(), SceneError>,
            {
                $(
                    if $name::REQUIRES_MUTABLE {
                        check(fields[$index])?;
                    }
                )+
                Ok(())
            }

            unsafe fn fetch(fields: &[*mut c_void]) -> Result<Self, SceneError> {
                Ok(($(
                    unsafe { $name::fetch(fields[$index])? },
                )+))
            }
        }
    };
}

impl_scene_query_tuple!(1; A: 0);
impl_scene_query_tuple!(2; A: 0, B: 1);
impl_scene_query_tuple!(3; A: 0, B: 1, C: 2);
impl_scene_query_tuple!(4; A: 0, B: 1, C: 2, D: 3);
impl_scene_query_tuple!(5; A: 0, B: 1, C: 2, D: 3, E: 4);
impl_scene_query_tuple!(6; A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_scene_query_tuple!(7; A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_scene_query_tuple!(8; A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);

impl_scene_query_mut_tuple!(1; A: 0);
impl_scene_query_mut_tuple!(2; A: 0, B: 1);
impl_scene_query_mut_tuple!(3; A: 0, B: 1, C: 2);
impl_scene_query_mut_tuple!(4; A: 0, B: 1, C: 2, D: 3);
impl_scene_query_mut_tuple!(5; A: 0, B: 1, C: 2, D: 3, E: 4);
impl_scene_query_mut_tuple!(6; A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_scene_query_mut_tuple!(7; A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_scene_query_mut_tuple!(8; A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);

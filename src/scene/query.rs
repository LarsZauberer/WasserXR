//! Typed component-field queries for `Scene::query` and `Scene::query_mut`.
//!
//! `QueryItem` maps `&T` to a shared field getter. `QueryItemMut` maps `&T`
//! to a shared getter and `&mut T` to a mutable getter. The `SceneQuery`
//! traits do the same for whole tuples by fetching each tuple item from the
//! matching entry in the `fields` slice.
//!
//! The raw `*mut c_void` is the internal escape hatch that lets one query
//! build several mutable-capable references from one component. `Scene::query_mut`
//! validates the field list first; after that, this module relies on the
//! component schema to return non-overlapping field pointers when mutable
//! references are involved.

use std::ffi::c_void;

use crate::{error::SceneError, scene::component::Component};

pub trait QueryItem<'scene>: Sized {
    unsafe fn fetch(component: *const c_void, field: &str) -> Result<Self, SceneError>;
}

impl<'scene, T: 'scene> QueryItem<'scene> for &'scene T {
    unsafe fn fetch(component: *const c_void, field: &str) -> Result<Self, SceneError> {
        let component = unsafe { &*(component as *const Component) };
        component
            .get::<T>(field)
            .map_err(SceneError::ComponentFieldError)
    }
}

pub trait QueryItemMut<'scene>: Sized {
    unsafe fn fetch(component: *mut c_void, field: &str) -> Result<Self, SceneError>;
}

impl<'scene, T: 'scene> QueryItemMut<'scene> for &'scene T {
    unsafe fn fetch(component: *mut c_void, field: &str) -> Result<Self, SceneError> {
        let component = unsafe { &*(component as *mut Component) };
        component
            .get::<T>(field)
            .map_err(SceneError::ComponentFieldError)
    }
}

impl<'scene, T: 'scene> QueryItemMut<'scene> for &'scene mut T {
    unsafe fn fetch(component: *mut c_void, field: &str) -> Result<Self, SceneError> {
        let component = unsafe { &mut *(component as *mut Component) };
        component
            .get_mut::<T>(field)
            .map_err(SceneError::ComponentFieldError)
    }
}

pub trait SceneQuery<'scene>: Sized {
    const FIELD_COUNT: usize;

    unsafe fn fetch(component: *const c_void, fields: &[&str]) -> Result<Self, SceneError>;
}

pub trait SceneQueryMut<'scene>: Sized {
    const FIELD_COUNT: usize;

    unsafe fn fetch(component: *mut c_void, fields: &[&str]) -> Result<Self, SceneError>;
}

macro_rules! impl_scene_query_tuple {
    ($count:expr; $($name:ident: $index:tt),+) => {
        impl<'scene, $($name),+> SceneQuery<'scene> for ($($name,)+)
        where
            $($name: QueryItem<'scene>),+
        {
            const FIELD_COUNT: usize = $count;

            unsafe fn fetch(
                component: *const c_void,
                fields: &[&str],
            ) -> Result<Self, SceneError> {
                Ok(($(
                    unsafe { $name::fetch(component, fields[$index])? },
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

            unsafe fn fetch(
                component: *mut c_void,
                fields: &[&str],
            ) -> Result<Self, SceneError> {
                Ok(($(
                    unsafe { $name::fetch(component, fields[$index])? },
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

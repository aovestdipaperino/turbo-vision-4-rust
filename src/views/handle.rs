// (C) 2026 - Enzo Lombardi

//! Typed handle to a child view.
//!
//! `GroupLike::add_typed` returns one, and `GroupLike::get` / `get_mut` turn it
//! back into the concrete child, replacing the
//! `child_by_id_mut().as_any_mut().downcast_mut::<T>()` dance. Borland reaches
//! children through typed pointers (`TInputLine* nameField`); this is the
//! ownership-safe equivalent.

use super::view::{View, ViewId};
use std::marker::PhantomData;

pub struct Handle<T: View>(ViewId, PhantomData<T>);

impl<T: View> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: View> Copy for Handle<T> {}

impl<T: View> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: View> Eq for Handle<T> {}

impl<T: View> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle<{}>({:?})", std::any::type_name::<T>(), self.0)
    }
}

impl<T: View> Handle<T> {
    /// Wrap an existing id. The type is a claim, checked by `GroupLike::get`
    /// at lookup time: a wrong type simply yields `None`.
    pub fn from_id(id: ViewId) -> Self {
        Self(id, PhantomData)
    }

    pub fn id(self) -> ViewId {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Rect;
    use crate::views::button::Button;
    use crate::views::group::{Group, GroupLike};
    use crate::views::input_line::InputLine;

    #[test]
    fn typed_handle_round_trips_through_a_group() {
        let mut g = Group::new(Rect::new(0, 0, 40, 10));
        let h = g.add_typed(InputLine::new(Rect::new(1, 1, 20, 2), 32));
        g.get_mut(h).unwrap().set_text("hello");
        assert_eq!(g.get(h).unwrap().text(), "hello");
        let wrong: Handle<Button> = Handle::from_id(h.id());
        assert!(g.get(wrong).is_none());
    }
}

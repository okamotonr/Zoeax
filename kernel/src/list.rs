use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

// TODO: Double LinkedList
#[derive(Debug)]
pub struct ListItem<T> {
    value: T,
    next: Option<NonNull<Self>>,
}

#[derive(Debug)]
pub struct LinkedList<T> {
    head: Option<NonNull<ListItem<T>>>,
    last: Option<NonNull<ListItem<T>>>,
}

impl<T> ListItem<T> {
    pub const fn new(value: T) -> Self {
        ListItem { value, next: None }
    }

    pub fn next_is_none(&self) -> bool {
        self.next.is_none()
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        LinkedList {
            head: None,
            last: None,
        }
    }

    pub fn push(&mut self, item: &mut ListItem<T>) {
        let ptr = unsafe { NonNull::new_unchecked(item as *mut ListItem<T>) };
        if let Some(prev_last) = &mut self.last.replace(ptr) {
            unsafe { prev_last.as_mut().next = Some(ptr) }
        } else {
            self.head = Some(ptr)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn pop<'a, 'b>(&'b mut self) -> Option<&'a mut ListItem<T>>
    where
        'a: 'b,
    {
        let result = self.head.take();
        let next = result.and_then(|mut ptr| unsafe { ptr.as_mut().next });
        if next.is_none() {
            self.last = None;
        }

        self.head = next;
        result.map(|ptr| unsafe { &mut *ptr.as_ptr() })
    }
}

impl<T> Deref for ListItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for ListItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(test)]
#[macro_use]
mod test {}

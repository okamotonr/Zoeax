use core::{marker::PhantomData, ptr::NonNull};
use core::ops::{DerefMut, Deref};

pub struct ListItem<'a, T> {
    value: T,
    next: Option<NonNull<ListItem<'a, T>>>,
    _marker: PhantomData<&'a T>,
}

pub struct LinkedList<'a, T> {
    head: Option<NonNull<ListItem<'a, T>>>,
    last: Option<NonNull<ListItem<'a, T>>>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> ListItem<'a, T> {
    pub const fn new(value: T) -> Self {
        ListItem {
            value,
            next: None,
            _marker: PhantomData
        }
    }

}

impl<'a, T> LinkedList<'a, T> {
    pub const fn new() -> Self {
        LinkedList {
            head: None,
            last: None,
            _marker: PhantomData
        }
    }

    pub fn push(&mut self, item: &'a mut ListItem<'a, T>) {
        let ptr = unsafe {
            NonNull::new_unchecked(item as *mut ListItem<T>)
        };
        if let Some(prev_last) = &mut self.last.replace(ptr) {
            unsafe {
                prev_last.as_mut().next = Some(ptr)
            }
        }
        else  {
            self.head = Some(ptr)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn pop(&mut self) -> Option<&'a mut ListItem<'a, T>> {
        let result = self.head.take();
        let next = result.and_then(|mut ptr| unsafe {
            ptr.as_mut().next
        });
        if next.is_none() {
            self.last = None;
        }

        self.head = next;
        result.map(|ptr| unsafe { &mut *ptr.as_ptr() })
    }

}

impl<'a, T> Deref for ListItem<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T> DerefMut for ListItem<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::ListItem;
    use super::LinkedList;

    #[test]
    fn test_run() {}
}

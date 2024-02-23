#![allow(unused)]
use std::ptr::NonNull;
use std::marker::PhantomData;
type Pointer<T> = Option<NonNull<Node<T>>>;/*
    Option< *const Node<T> >, an nullable pointer to Node<T>
*/
struct Node<T> {
    front:Pointer<T>,
    next:Pointer<T>,
    data:Option<T>
}
impl<T> Node<T> {
    fn new_raw(data:T) -> Pointer<T> {
        let ptr = Box::into_raw(Box::new(
            Self {
                front:None,
                next:None,
                data:Some(data)
            }
        ));
        return NonNull::new(ptr)
    }
}
pub struct LinkedList <T>{
    head:Pointer<T>,
    tail:Pointer<T>,
    len:usize,
    _m:PhantomData<T>
}
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head:None,
            tail:None,
            len:0,
            _m:PhantomData
        }
    }
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn peek_head(&self) -> Option<&T> {
        if self.len() == 0 {return None}
        let ptr = self.head?;
        unsafe {
            return (&ptr.as_ref().data).as_ref()
        }
    }
    pub fn peek_tail(&self) -> Option<&T> {
        if self.len() == 0 {return None}
        let ptr = self.tail?;
        unsafe {
            return (&ptr.as_ref().data).as_ref()
        }
    }
    pub fn peek_head_mut(&mut self) -> Option<&mut T> {
        if self.len() == 0 {return None}
        let mut ptr = self.head?;
        unsafe {
            return (&mut ptr.as_mut().data).as_mut()
        }
    }
    pub fn peek_tail_mut(&mut self) -> Option<&mut T> {
        if self.len() == 0 {return None}
        let mut ptr = self.tail?;
        unsafe {
            return (&mut ptr.as_mut().data).as_mut()
        }
    }
    pub fn push_head(&mut self,data:T) {
        let _ = self.push_head_result(data).expect("Insertion failed");
    }
    pub fn push_tail(&mut self,data:T) {
        let _ = self.push_tail_result(data).expect("Insertion failed");
    }
    #[inline]
    fn push_head_result(&mut self,data:T) -> Result<(),&str>{
        //to prevent panic, operations that may cause panic are done outside the LinkedList
        let mut ptr = Node::new_raw(data).ok_or("Allocation Failed")?;//this may panic if box allocation fails
        
        if self.len() == 0 {
            //there must be no panic next or the list will be damaged
            self.head = Some(ptr);
            self.tail = Some(ptr);
            self.len = 1;
            return Ok(());
        }

        //guard prerequisit, return if condition not met
        let mut old_head = self.head.ok_or("Broken list")?;
        let new_size = self.len + 1; // this may fail if overflow, do the calculation before touches list
        //true modifaction happens, no panic allowed
        let _ = self.head.take();
        unsafe {
            ptr.as_mut().next = Some(old_head);
            old_head.as_mut().front = Some(ptr);
            self.head = Some(ptr);
        }
        self.len = new_size;
        return Ok(())
    }
    #[inline]
    fn push_tail_result(&mut self,data:T) -> Result<(),&str> {
        //to prevent panic, operations that may cause panic are done outside the LinkedList
        let mut ptr = Node::new_raw(data).ok_or("Allocation Failed")?;//this may panic if box allocation fails
        
        if self.len() == 0 {
            //there must be no panic next or the list will be damaged
            self.head = Some(ptr);
            self.tail = Some(ptr);
            self.len = 1;
            return Ok(());
        }
        let old_tail = self.tail;
        let mut old_tail = old_tail.ok_or("Broken list")?;
        let new_size = self.len + 1; // this may fail if overflow, do the calculation before touches list
        //modifing list, no panic allowed
        let _ = self.tail.take();
        unsafe {
            old_tail.as_mut().next = Some(ptr);
            ptr.as_mut().front = Some(old_tail);
        }
        self.tail = Some(ptr);
        self.len = new_size;
        return Ok(())
    }
    pub fn pop_head(&mut self) -> Option<T> {
        if self.len() == 0 {
            self.head = None;
            self.tail = None;
            return None;
        } 
        if self.len() == 1 {
            self.len = 0;
            self.tail = None;
            let mut data = None;
            if let Some(p) = self.head.take() { // even if box::from_raw failed, the list is still valid
                let mut boxed_node = unsafe {Box::from_raw(p.as_ptr())};
                data = boxed_node.data.take();
            }
            return data;
        } 
        let mut head = self.head?;
        let mut head_next = unsafe {head.as_mut().next?};
        //when len > 1, both head and head's next must exist, if not, aborting , don't touch the list
        let mut data = None;
        unsafe {
            head_next.as_mut().front = None;//the new head shall have no front nodes
            head.as_mut().next = None;//just to be sure head->front and head->next is None, not significant, just to be safe
            self.head = Some(head_next);
            self.len -= 1;
            //even if the next panics the list is still valid
            let mut boxed_node = Box::from_raw(head.as_ptr());
            data = boxed_node.data.take();
        }
        return data;
    }
    pub fn pop_tail(&mut self) -> Option<T> {
        if self.len() == 0 {
            self.head = None;
            self.tail = None;
            return None;
        } 
        if self.len() == 1 {
            self.len = 0;
            self.head = None;
            let mut data = None;
            if let Some(p) = self.tail.take() {// even if box::from_raw failed, the list is still valid
                let mut boxed_node = unsafe {Box::from_raw(p.as_ptr())};
                data = boxed_node.data.take();
            }
            return data;
        } 
        let mut tail = self.tail?;
        let mut tail_front = unsafe {tail.as_mut().front?};
        let mut data = None;
        unsafe {
            tail_front.as_mut().next = None;
            tail.as_mut().front = None;
            self.tail = Some(tail_front);
            self.len -= 1;
            let mut boxed_node = Box::from_raw(tail.as_ptr());
            data = boxed_node.data.take();
        }
        return data;
    }
    pub fn cursor_head(&self) -> Cursor<'_,T>{
        let mut cursor = Cursor::new(self);
        if self.len() == 0 || self.head.is_none() {return cursor;}
        cursor.index = Some(0);
        cursor.node_ptr = self.head;
        return cursor;
    }
    pub fn cursor_tail(&self) -> Cursor<'_,T>{
        let mut cursor = Cursor::new(self);
        if self.len() == 0 || self.tail.is_none() {return cursor;}
        cursor.index = Some(self.len()-1);
        cursor.node_ptr = self.tail;
        return cursor;
    }
    pub fn cursor_mut_head(&mut self) -> CursorMut<'_,T>{
        let len = self.len();
        let head = self.head;
        let mut cursor = CursorMut::new(self);
        if len == 0 || head.is_none() {return cursor;}
        cursor.index = Some(0);
        cursor.node_ptr = head;
        return cursor;
    }
    pub fn cursor_mut_tail(&mut self) -> CursorMut<'_,T>{
        let len = self.len();
        let tail = self.tail;
        let mut cursor = CursorMut::new(self);
        if len == 0 || tail.is_none() {return cursor;}
        cursor.index = Some(len-1);
        cursor.node_ptr = tail;
        return cursor;
    }
    pub fn iter(&self) -> Iter<'_,T> {
        Iter {
            ptr:self.head,
            borrow:self
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<'_,T> {
        IterMut {
            ptr:self.head,
            borrow:self
        }
    }
}

use std::ops::Drop;
impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.len() > 0 {
            let _ = self.pop_head();
        }
    }
}

pub struct Cursor<'a,T> {
    node_ptr:Pointer<T>,
    index:Option<usize>,
    borrow:&'a LinkedList<T>
}
type SizeAndNext<T> = Option<(usize,NonNull<Node<T>>)>;
impl<'a,T> Cursor<'a,T> {
    fn new(borrow:&'a LinkedList<T>) -> Self {
        Self {
            node_ptr:None,
            index:None,
            borrow:borrow
        }
    }
    pub fn is_empty(&self) -> bool {
        self.borrow.len() == 0
    }
    pub fn index(&self) -> Option<usize> {
        self.index
    }
    #[inline]
    fn find_next(&self) -> SizeAndNext<T> {
        let index = self.index?;
        if index >= self.borrow.len() - 1 {return None}
        let next_ptr = unsafe{self.node_ptr?.as_ref()}.next?;
        return Some((index,next_ptr))
    }
    #[inline]
    fn find_front(&self) -> SizeAndNext<T> {
        let index = self.index?;
        if index == 0 {return None}
        let front_ptr = unsafe{self.node_ptr?.as_ref()}.front?;
        return Some((index,front_ptr))
    }
    pub fn move_next(&mut self) {
        if self.is_empty() {
            self.node_ptr = None;
            self.index = None;
            return;
        }
        let (index,next_ptr) = match self.find_next() {
            None => {
                self.index = Some(0);
                self.node_ptr = self.borrow.head;
                return;
            }
            Some(t) => {t}
        };
        self.node_ptr = Some(next_ptr);
        self.index = Some(index + 1);
    }
    pub fn move_front(&mut self) {
        if self.is_empty() {
            self.node_ptr = None;
            self.index = None;
            return;
        }
        let (index,front_ptr) = match self.find_front() {
            Some(t) => t,
            None => {
                self.index = Some(self.borrow.len()-1);
                self.node_ptr = self.borrow.tail;
                return;
            }
        };
        self.node_ptr = Some(front_ptr);
        self.index = Some(index - 1);
    }
    pub fn peek(&self) -> Option<&T> {
        let ptr = self.node_ptr?;
        let _index = self.index?;
        unsafe {
            return (&ptr.as_ref().data).as_ref();
        }
    }
}

pub struct CursorMut<'a,T> {
    node_ptr:Pointer<T>,
    index:Option<usize>,
    borrow_mut:&'a mut LinkedList<T>
}

impl<'a,T> CursorMut<'a,T> {
    fn new(mut_borrow:&'a mut LinkedList<T>) -> Self {
        Self {
            node_ptr:None,
            index:None,
            borrow_mut:mut_borrow
        }
    }
    pub fn index(&self) -> Option<usize> {
        self.index
    }
    fn is_empty(&self) -> bool {
        self.borrow_mut.is_empty()
    }
    #[inline]
    fn find_next(&self) -> SizeAndNext<T> {
        let index = self.index?;
        if index >= self.borrow_mut.len() - 1 {return None}
        let next_ptr = unsafe{self.node_ptr?.as_ref()}.next?;
        return Some((index,next_ptr))
    }
    #[inline]
    fn find_front(&self) -> SizeAndNext<T> {
        let index = self.index?;
        if index == 0 {return None}
        let front_ptr = unsafe{self.node_ptr?.as_ref()}.front?;
        return Some((index,front_ptr))
    }
    pub fn move_next(&mut self) {
        if self.is_empty() {
            self.node_ptr = None;
            self.index = None;
            return;
        }
        let (index,next_ptr) = match self.find_next() {
            Some(t) => t,
            None => {
                self.index = Some(0);
                self.node_ptr = self.borrow_mut.head;
                return;
            }
        };
        self.node_ptr = Some(next_ptr);
        self.index = Some(index + 1);
    }
    pub fn move_front(&mut self) {
        if self.is_empty() {
            self.node_ptr = None;
            self.index = None;
            return;
        }
        let (index,front_ptr) = match self.find_front() {
            Some(t) => t,
            None => {
                self.index = Some(self.borrow_mut.len()-1);
                self.node_ptr = self.borrow_mut.tail;
                return;
            }
        };
        self.node_ptr = Some(front_ptr);
        self.index = Some(index - 1);
    }
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        (&mut unsafe {self.node_ptr?.as_mut()}.data).as_mut()
    }
    pub fn push(&mut self, data:T) {
        if self.is_empty() {
            self.borrow_mut.push_head(data);
            self.index = Some(0);
            self.node_ptr = self.borrow_mut.head;
            return ;
        }
        let (_,mut next_ptr) = match self.find_next() {
            Some(t) => t,
            None => {
                self.borrow_mut.push_tail(data);
                return;
            }
        };
        //may panic happens here
        let mut to_insert = Node::new_raw(data).unwrap();
        let mut current = self.node_ptr.unwrap();
        let new_len = self.borrow_mut.len + 1;
        //no panic allowed
        unsafe {
            current.as_mut().next = Some(to_insert);
            to_insert.as_mut().next = Some(next_ptr);
            next_ptr.as_mut().front = Some(to_insert);
            to_insert.as_mut().front = Some(current);
        }
        self.borrow_mut.len = new_len;
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.borrow_mut.len() <= 1 {
            self.node_ptr = None;
            self.index = None;
            return self.borrow_mut.pop_head();
        }
        let mut current_ptr = self.node_ptr?;
        let (index,mut front_ptr) = match self.find_front() {
            Some(t) => t,
            None => {
                let data = self.borrow_mut.pop_head();
                self.index = Some(0);
                self.node_ptr = self.borrow_mut.head;
                return data;
            }
        };
        let (_,mut next_ptr) = match self.find_next() {
            Some(t) => t,
            None => {
                let data = self.borrow_mut.pop_tail();
                self.index = Some(self.borrow_mut.len()-1);
                self.node_ptr = self.borrow_mut.tail;
                return data;
            }
        };
        let len = self.borrow_mut.len() - 1;
        let mut data = None;

        //touching the link
        self.borrow_mut.len = len;
        unsafe {
            current_ptr.as_mut().next = None;
            current_ptr.as_mut().front = None;
            next_ptr.as_mut().front = Some(front_ptr);
            front_ptr.as_mut().next = Some(next_ptr);
            data = Box::from_raw(current_ptr.as_ptr()).data.take();
        }
        self.node_ptr = Some(front_ptr);
        self.index = Some(index - 1);
        return data;
    }
}

pub struct Iter<'a,T:'a> {
    ptr:Pointer<T>,
    borrow:&'a LinkedList<T>
}

use std::iter::{Iterator,IntoIterator};
impl<'a,T> Iterator for Iter<'a,T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.ptr?;
        self.ptr = unsafe{ptr.as_ref().next};
        return unsafe {ptr.as_ref().data.as_ref()}
    }
}

pub struct IterMut<'a,T> {
    ptr:Pointer<T>,
    borrow:&'a mut LinkedList<T>
}

impl<'a,T> Iterator for IterMut<'a,T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut ptr = self.ptr?;
        self.ptr = unsafe{ptr.as_ref().next};
        return unsafe {ptr.as_mut().data.as_mut()}
    }
}

#[repr(transparent)]
pub struct IntoIter<T> {
    list:LinkedList<T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_head()
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            list:self
        }
    }
}

use std::marker::{Send,Sync};

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

/* I am not sure
unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}
*/
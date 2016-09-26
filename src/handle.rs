use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

pub trait HandleSpecifier {
    type Raw: Copy;
}

pub struct Handle<HS: HandleSpecifier>(HS::Raw, PhantomData<HS>);
impl<HS: HandleSpecifier> Handle<HS> {
    pub unsafe fn new(raw: HS::Raw) -> Self {
        Handle(raw, PhantomData)
    }
    pub fn raw(&self) -> HS::Raw {
        self.0
    }
}

pub trait HandleBorrow<HS: HandleSpecifier> {
    fn borrow(&self) -> &Handle<HS>;
}
impl<HS: HandleSpecifier> HandleBorrow<HS> for Handle<HS> {
    fn borrow(&self) -> &Handle<HS> {
        self
    }
}
impl<'hb, HS: HandleSpecifier, HB: HandleBorrow<HS>> HandleBorrow<HS> for &'hb HB {
    fn borrow(&self) -> &Handle<HS> {
        (**self).borrow()
    }
}
impl<'hb, HS: HandleSpecifier, HB: HandleBorrow<HS>> HandleBorrow<HS> for &'hb mut HB {
    fn borrow(&self) -> &Handle<HS> {
        (**self).borrow()
    }
}
impl<HS: HandleSpecifier, HB: HandleBorrow<HS>> HandleBorrow<HS> for Rc<HB> {
    fn borrow(&self) -> &Handle<HS> {
        (**self).borrow()
    }
}
impl<HS: HandleSpecifier, HB: HandleBorrow<HS>> HandleBorrow<HS> for Arc<HB> {
    fn borrow(&self) -> &Handle<HS> {
        (**self).borrow()
    }
}

pub trait HandleBorrowMut<HS: HandleSpecifier>: HandleBorrow<HS> {
    fn borrow_mut(&mut self) -> &mut Handle<HS>;
}
impl<HS: HandleSpecifier> HandleBorrowMut<HS> for Handle<HS> {
    fn borrow_mut(&mut self) -> &mut Handle<HS> {
        self
    }
}
impl<'hb, HS: HandleSpecifier, HB: HandleBorrowMut<HS>> HandleBorrowMut<HS> for &'hb mut HB {
    fn borrow_mut(&mut self) -> &mut Handle<HS> {
        (**self).borrow_mut()
    }
}

use std::cmp;

#[derive(Debug, Clone, Copy)]
pub struct UnOrd<T>(pub T);

impl<T> From<T> for UnOrd<T> {
    fn from(value: T) -> Self { Self(value) }
}

impl<T> PartialEq for UnOrd<T> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<T> Eq for UnOrd<T> {}
impl<T> PartialOrd for UnOrd<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<cmp::Ordering> { Some(cmp::Ordering::Equal) }
}
impl<T> Ord for UnOrd<T> {
    fn cmp(&self, _other: &Self) -> cmp::Ordering { cmp::Ordering::Equal }
}

impl<T: screeps::HasPosition> screeps::HasPosition for UnOrd<T> {
    #[doc = " Position of the object."]
    fn pos(&self) -> screeps::Position {
        self.0.pos()
    }
}
impl< T: screeps::HasTypedId<T> > screeps::HasTypedId<T> for UnOrd<T> {
        #[doc = " Object ID of the object, which can be used to efficiently fetch a"]
    #[doc = " fresh reference to the object on subsequent ticks."]
    fn id(&self) -> screeps::ObjectId<T> {
        self.0.id()
    }
    fn js_id(&self) -> screeps::JsObjectId<T> {
        self.0.js_id()
    }
}
impl< T: screeps::MaybeHasTypedId<T> > screeps::MaybeHasTypedId<T> for UnOrd<T> {
    #[doc = " Object ID of the object, which can be used to efficiently fetch a"]
    #[doc = " fresh reference to the object on subsequent ticks, or `None` if the"]
    #[doc = " object doesn\'t currently have an id."]
    fn try_id(&self) -> Option<screeps::ObjectId<T>> {
        self.0.try_id()
    }
}

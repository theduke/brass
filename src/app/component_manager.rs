use super::component::InstantiatedComponent;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ComponentId {
    index: u32,
    revision: u32,
}

impl ComponentId {
    pub const NONE: Self = Self {
        index: 0,
        revision: 0,
    };

    pub const ROOT: Self = Self {
        index: 1,
        revision: 0,
    };

    pub fn is_none(self) -> bool {
        self == Self::NONE
    }

    // pub fn is_root(self) -> bool {
    //     self == Self::ROOT
    // }
}

#[must_use]
pub(crate) struct ComponentBorrowFinisher {
    id: ComponentId,
}

impl ComponentBorrowFinisher {
    pub fn return_component(
        self,
        manager: &mut ComponentManager,
        comp: Box<InstantiatedComponent>,
    ) {
        manager.return_borrowed(self.id, comp);
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }
}

pub(crate) struct ComponentManager {
    components: Vec<Option<Box<InstantiatedComponent>>>,
    idle: Vec<ComponentId>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            idle: Vec::new(),
        }
    }

    // pub fn register_component(&mut self, state: InstantiatedComponent) -> ComponentId {
    //     if let Some(old_id) = self.idle.pop() {
    //         self.components[Self::id_to_index(old_id)] = Some(state);
    //         old_id
    //     } else {
    //         let id = Self::index_to_id(self.components.len());
    //         self.components.push(Some(state));
    //         id
    //     }
    // }

    pub fn reserve_id(&mut self) -> ComponentBorrowFinisher {
        let id = if let Some(mut old_id) = self.idle.pop() {
            old_id.revision += 1;
            old_id
        } else {
            let id = ComponentId {
                index: self.components.len() as u32 + 1,
                revision: 0,
            };
            self.components.push(None);
            id
        };
        ComponentBorrowFinisher { id }
    }

    // pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut InstantiatedComponent> {
    //     let index = (id.index - 1) as usize;
    //     let c = self.components.get_mut(index)?.as_mut()?;
    //     let existing_id = c.state_mut().id();
    //     if existing_id.revision == id.revision {
    //         Some(c)
    //     } else {
    //         None
    //     }
    // }

    pub fn borrow(
        &mut self,
        id: ComponentId,
    ) -> Option<(Box<InstantiatedComponent>, ComponentBorrowFinisher)> {
        let index = id.index - 1;
        let c = self.components.get_mut(index as usize)?.take()?;
        Some((c, ComponentBorrowFinisher { id }))
    }

    fn return_borrowed(&mut self, id: ComponentId, comp: Box<InstantiatedComponent>) {
        let index = (id.index - 1) as usize;
        *self
            .components
            .get_mut(index)
            .expect("Invalid component id") = Some(comp);
    }

    pub fn remove(&mut self, id: ComponentId) -> Option<InstantiatedComponent> {
        let index = (id.index - 1) as usize;
        let old = self.components.get_mut(index).and_then(|x| x.take())?;
        self.idle.push(id);
        Some(*old)
    }
}

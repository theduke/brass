use super::component::InstantiatedComponent;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ComponentId(u32);

impl ComponentId {
    pub const NONE: Self = Self(0);
    pub const ROOT: Self = Self(1);

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
    pub fn return_component(self, manager: &mut ComponentManager, comp: InstantiatedComponent) {
        manager.components[ComponentManager::id_to_index(self.id)] = Some(comp);
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }
}

pub(crate) struct ComponentManager {
    components: Vec<Option<InstantiatedComponent>>,
    idle: Vec<ComponentId>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            idle: Vec::new(),
        }
    }

    #[inline]
    fn id_to_index(id: ComponentId) -> usize {
        id.0 as usize - 1
    }

    #[inline]
    fn index_to_id(index: usize) -> ComponentId {
        ComponentId(index as u32 + 1)
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
        let id = if let Some(old_id) = self.idle.pop() {
            old_id
        } else {
            let id = Self::index_to_id(self.components.len());
            self.components.push(None);
            id
        };
        ComponentBorrowFinisher { id }
    }

    pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut InstantiatedComponent> {
        let real_id = Self::id_to_index(id);
        self.components.get_mut(real_id).and_then(|x| x.as_mut())
    }

    pub fn remove(&mut self, id: ComponentId) -> Option<InstantiatedComponent> {
        let index = Self::id_to_index(id);
        self.idle.push(id);
        self.components.get_mut(index).and_then(|x| x.take())
    }
}

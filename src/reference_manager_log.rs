use std::{collections::HashMap, fmt::Debug};

use moirai_crdt::policy::LwwPolicy;
use moirai_protocol::{
    clock::version_vector::Version,
    event::{Event, id::EventId, tagged_op::TaggedOp},
    state::{po_log::POLog, unstable_state::IsUnstableState},
};

use crate::references::{Instance, ReferenceManager, Refs};
use moirai_protocol::state::sink::ObjectPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReferenceManagerDerivedKey {
    AddVertex(Instance),
    RemoveVertex(Instance),
    DeleteSubtree(ObjectPath),
    AddArc(Refs),
    RemoveArc(Refs),
}

#[derive(Debug, Clone, Default)]
pub struct ReferenceManagerState {
    ops: HashMap<(EventId, ReferenceManagerDerivedKey), TaggedOp<ReferenceManager<LwwPolicy>>>,
    order: Vec<(EventId, ReferenceManagerDerivedKey)>,
}

impl IsUnstableState<ReferenceManager<LwwPolicy>> for ReferenceManagerState {
    type Key = (EventId, ReferenceManagerDerivedKey);

    fn append(&mut self, event: Event<ReferenceManager<LwwPolicy>>) {
        let tagged_op = TaggedOp::from(&event);
        let key = self.key_of(&tagged_op);
        self.order.push(key.clone());
        self.ops.insert(key, tagged_op);
    }

    fn key_of(&self, tagged_op: &TaggedOp<ReferenceManager<LwwPolicy>>) -> Self::Key {
        let derived = match tagged_op.op() {
            ReferenceManager::AddVertex { id } => ReferenceManagerDerivedKey::AddVertex(id.clone()),
            ReferenceManager::RemoveVertex { id } => {
                ReferenceManagerDerivedKey::RemoveVertex(id.clone())
            }
            ReferenceManager::DeleteSubtree { prefix } => {
                ReferenceManagerDerivedKey::DeleteSubtree(prefix.clone())
            }
            ReferenceManager::AddArc(arc) => ReferenceManagerDerivedKey::AddArc(arc.clone()),
            ReferenceManager::RemoveArc(arc) => ReferenceManagerDerivedKey::RemoveArc(arc.clone()),
            _ => unreachable!(),
        };
        (tagged_op.id().clone(), derived)
    }

    fn get(&self, event_id: &EventId) -> Option<&TaggedOp<ReferenceManager<LwwPolicy>>> {
        self.order
            .iter()
            .find(|(id, _)| id == event_id)
            .and_then(|key| self.ops.get(key))
    }

    fn get_by_key(&self, key: &Self::Key) -> Option<&TaggedOp<ReferenceManager<LwwPolicy>>> {
        self.ops.get(key)
    }

    fn remove(&mut self, event_id: &EventId) {
        if let Some(pos) = self.order.iter().position(|(id, _)| id == event_id) {
            let key = self.order.remove(pos);
            self.ops.remove(&key);
        }
    }

    fn remove_by_key(&mut self, key: &Self::Key) {
        self.ops.remove(key);
        if let Some(pos) = self.order.iter().position(|existing| existing == key) {
            self.order.remove(pos);
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a TaggedOp<ReferenceManager<LwwPolicy>>>
    where
        ReferenceManager<LwwPolicy>: 'a,
    {
        self.order.iter().filter_map(|key| self.ops.get(key))
    }

    fn retain<T: Fn(&TaggedOp<ReferenceManager<LwwPolicy>>) -> bool>(&mut self, predicate: T) {
        self.order.retain(|key| match self.ops.get(key) {
            Some(tagged_op) if predicate(tagged_op) => true,
            Some(_) => {
                self.ops.remove(key);
                false
            }
            None => false,
        });
    }

    fn len(&self) -> usize {
        self.ops.len()
    }

    fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    fn clear(&mut self) {
        self.ops.clear();
        self.order.clear();
    }

    fn predecessors(&self, version: &Version) -> Vec<TaggedOp<ReferenceManager<LwwPolicy>>> {
        self.iter()
            .filter(|tagged_op| tagged_op.id().is_predecessor_of(version))
            .cloned()
            .collect()
    }

    fn parents(&self, _event_id: &EventId) -> Vec<EventId> {
        unimplemented!()
    }

    fn delivery_order(&self, event_id: &EventId) -> usize {
        self.order
            .iter()
            .position(|(id, _)| id == event_id)
            .unwrap()
    }

    fn frontier(&self) -> Vec<TaggedOp<ReferenceManager<LwwPolicy>>> {
        self.iter().cloned().collect()
    }
}

pub type ReferenceManagerLog = POLog<ReferenceManager<LwwPolicy>, ReferenceManagerState>;

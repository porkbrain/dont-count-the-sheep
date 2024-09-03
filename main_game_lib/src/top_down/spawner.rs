//! Use [TopDownTsncSpawner] to spawn tscn files in top down scenes.

use std::str::FromStr;

use bevy::utils::EntityHashMap;
use rscn::{EntityDescription, NodeName, RscnNode, TscnSpawnHooks};
use top_down::{
    inspect_and_interact::ZoneToInspectLabelEntity, InspectLabelCategory,
    TopDownAction, ZoneTileKind,
};

use crate::{prelude::*, vec2_ext::Vec2Ext};

/// The implementation has some knowledge of top down scenes to provide
/// default implementations for things like [`crate::top_down::InspectLabel`]
/// that maps actions `A` to labels and Y sorting.
///
/// Other nodes are delegated to the user implementation `T`.
pub struct TopDownTsncSpawner<'a, T> {
    inner: &'a mut T,
    zone_to_inspect_label_entity: &'a mut ZoneToInspectLabelEntity,
}

impl<'a, T> TopDownTsncSpawner<'a, T> {
    /// Create a new top down spawner with user implementation `T`.
    pub fn new(
        zone_to_inspect_label_entity: &'a mut ZoneToInspectLabelEntity,
        inner: &'a mut T,
    ) -> Self {
        Self {
            inner,
            zone_to_inspect_label_entity,
        }
    }
}

impl<'a, T> TscnSpawnHooks for TopDownTsncSpawner<'a, T>
where
    T: TscnSpawnHooks,
{
    fn handle_2d_node(
        &mut self,
        cmd: &mut Commands,
        descriptions: &mut EntityHashMap<Entity, EntityDescription>,
        parent: Option<(Entity, NodeName)>,
        (entity, NodeName(name)): (Entity, NodeName),
    ) {
        match name.as_str() {
            // Given a position in 2D, add a z index to it.
            // This function is used for those nodes that don't have a z index
            // set. If a 2D node has a 2D node child called "YSort",
            // then the position fed to this function is the global
            // position of that "YSort" node.
            "YSort" => {
                let (parent, _) = parent.expect("YSort must have a parent");
                // this will despawn the YSort node
                let own_description = descriptions
                    .remove(&entity)
                    .expect("YSort must be a 2D node with description");
                if let Some(parent_description) = descriptions.get_mut(&parent)
                {
                    parent_description.z_index = Some(
                        (parent_description.translation
                            + own_description.translation)
                            .ysort(),
                    );
                }
            }
            _ => {
                self.inner.handle_2d_node(
                    cmd,
                    descriptions,
                    parent,
                    (entity, NodeName(name)),
                );
            }
        }
    }

    fn handle_plain_node(
        &mut self,
        cmd: &mut Commands,
        descriptions: &mut EntityHashMap<Entity, EntityDescription>,
        (parent_entity, parent_name): (Entity, NodeName),
        (NodeName(name), mut plain_node): (NodeName, RscnNode),
    ) {
        match name.as_str() {
            "InspectLabel" => {
                let with_label = plain_node.metadata.remove("label").expect(
                    "Label metadata must be present on InspectLabelCategory",
                );

                let mut label = plain_node
                    .metadata
                    .remove("category")
                    .map(|cat| {
                        InspectLabelCategory::from_str(&cat).expect(
                            "category must be a valid InspectLabelCategory",
                        )
                    })
                    .unwrap_or_default()
                    .into_label(with_label);

                if let Some(action) = plain_node.metadata.remove("action") {
                    label.set_emit_event_on_interacted(
                        TopDownAction::from_str(&action).unwrap_or_else(|_| {
                            panic!("InspectLabel action '{action}' not valid")
                        }),
                    );
                }

                cmd.entity(parent_entity).insert(label);

                if let Some(zone) = plain_node.metadata.remove("zone") {
                    self.zone_to_inspect_label_entity.insert(
                        ZoneTileKind::from_str(&zone).unwrap_or_else(|_| {
                            panic!(
                                "Zone '{zone}' not valid for InspectLabel of {name:?}"
                            )
                        }),
                        parent_entity,
                    );
                }

                assert!(
                    plain_node.metadata.is_empty(),
                    "InspectLabel node can only have \
                    label, category, action and zone metadata"
                );
            }
            _ => {
                self.inner.handle_plain_node(
                    cmd,
                    descriptions,
                    (parent_entity, parent_name),
                    (NodeName(name), plain_node),
                );
            }
        }
    }
}

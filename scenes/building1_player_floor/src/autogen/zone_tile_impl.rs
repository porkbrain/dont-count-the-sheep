#[rustfmt::skip]
/// This is an autogenerated implementation by the map maker tool.
impl main_game_lib::top_down::layout::ZoneTile for crate::Building1PlayerFloorTileKind
{ #[inline] fn zone_group(&self) ->
Option<main_game_lib::top_down::layout::ZoneGroup>
{
    use main_game_lib::top_down::layout::ZoneGroup;
    #[allow(clippy::match_single_binding)] match self
    { Self::HallwayZone => Some(ZoneGroup(0)),
Self::PlayerApartmentZone => Some(ZoneGroup(0)),
Self::BedZone => Some(ZoneGroup(0)),
Self::ElevatorZone => Some(ZoneGroup(0)),
Self::PlayerDoorZone => Some(ZoneGroup(0)),
Self::MeditationZone => Some(ZoneGroup(0)),
Self::TeaZone => Some(ZoneGroup(0)),
 #[allow(unreachable_patterns)] _ => None, }
} #[inline] fn zone_size(&self) -> Option<usize>
{
    #[allow(clippy::match_single_binding)] match self
    { Self::HallwayZone => Some(1266),
Self::PlayerApartmentZone => Some(2725),
Self::BedZone => Some(30),
Self::ElevatorZone => Some(68),
Self::PlayerDoorZone => Some(64),
Self::MeditationZone => Some(125),
Self::TeaZone => Some(45),
 #[allow(unreachable_patterns)] _ => None, }
} type Successors = Self; #[inline] fn zone_successors(&self) -> Option<&'static
[Self::Successors]>
{
    #[allow(clippy::match_single_binding)] match self
    { Self::HallwayZone => Some(&[Self::PlayerApartmentZone,Self::ElevatorZone,Self::PlayerDoorZone]),
Self::PlayerApartmentZone => Some(&[Self::HallwayZone,Self::BedZone,Self::PlayerDoorZone,Self::MeditationZone,Self::TeaZone]),
Self::BedZone => Some(&[Self::PlayerApartmentZone]),
Self::ElevatorZone => Some(&[Self::HallwayZone]),
Self::PlayerDoorZone => Some(&[Self::HallwayZone,Self::PlayerApartmentZone]),
Self::MeditationZone => Some(&[Self::PlayerApartmentZone]),
Self::TeaZone => Some(&[Self::PlayerApartmentZone]),
 #[allow(unreachable_patterns)] _ => None, }
}
 }

#[rustfmt::skip]
/// This is an autogenerated implementation by the map maker tool.
impl main_game_lib::top_down::layout::ZoneTile for crate::MallTileKind
{ #[inline] fn zone_group(&self) ->
Option<main_game_lib::top_down::layout::ZoneGroup>
{
    use main_game_lib::top_down::layout::ZoneGroup;
    #[allow(clippy::match_single_binding)] match self
    { Self::ExitZone => Some(ZoneGroup(0)),
 #[allow(unreachable_patterns)] _ => None, }
} #[inline] fn zone_size(&self) -> Option<usize>
{
    #[allow(clippy::match_single_binding)] match self
    { Self::ExitZone => Some(153),
 #[allow(unreachable_patterns)] _ => None, }
} type Successors = Self; #[inline] fn zone_successors(&self) -> Option<&'static
[Self::Successors]>
{
    #[allow(clippy::match_single_binding)] match self
    {  #[allow(unreachable_patterns)] _ => None, }
}
 }

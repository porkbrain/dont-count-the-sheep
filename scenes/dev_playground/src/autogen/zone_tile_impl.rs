/// This is an autogenerated implementation by the map maker tool.
#[rustfmt::skip] impl common_top_down::layout::ZoneTile for crate::layout::DevPlaygroundTileKind
{ #[inline] fn zone_group(&self) -> Option<common_top_down::layout::ZoneGroup>
{
    use common_top_down::layout::ZoneGroup; match self
    { Self::ZoneA => Some(ZoneGroup(0)),
Self::ZoneB => Some(ZoneGroup(0)),
Self::ZoneC => Some(ZoneGroup(1)),
Self::ZoneD => Some(ZoneGroup(0)),
Self::ZoneE => Some(ZoneGroup(0)),
Self::ZoneF => Some(ZoneGroup(0)),
Self::ZoneG => Some(ZoneGroup(0)),
Self::ZoneH => Some(ZoneGroup(1)),
Self::ZoneI => Some(ZoneGroup(0)),
Self::ZoneJ => Some(ZoneGroup(0)),
Self::ZoneK => Some(ZoneGroup(0)),
 #[allow(unreachable_patterns)] _ => None, }
} #[inline] fn zone_size(&self) -> Option<usize>
{ match self { Self::ZoneA => Some(360),
Self::ZoneB => Some(308),
Self::ZoneC => Some(380),
Self::ZoneD => Some(112),
Self::ZoneE => Some(35),
Self::ZoneF => Some(4),
Self::ZoneG => Some(72),
Self::ZoneH => Some(64),
Self::ZoneI => Some(55),
Self::ZoneJ => Some(12),
Self::ZoneK => Some(2),
 #[allow(unreachable_patterns)] _ => None, } } type Successors = Self; #[inline] fn zone_successors(&self) -> Option<&'static
[Self::Successors]>
{ match self { Self::ZoneA => Some(&[Self::ZoneB,Self::ZoneD,Self::ZoneE,Self::ZoneF,Self::ZoneG,Self::ZoneI,Self::ZoneJ,Self::ZoneK]),
Self::ZoneB => Some(&[Self::ZoneA,Self::ZoneG,Self::ZoneI]),
Self::ZoneC => Some(&[Self::ZoneH]),
Self::ZoneD => Some(&[Self::ZoneA,Self::ZoneE,Self::ZoneF,Self::ZoneG]),
Self::ZoneE => Some(&[Self::ZoneA,Self::ZoneD,Self::ZoneF]),
Self::ZoneF => Some(&[Self::ZoneA,Self::ZoneD,Self::ZoneE]),
Self::ZoneG => Some(&[Self::ZoneA,Self::ZoneB,Self::ZoneD,Self::ZoneI]),
Self::ZoneH => Some(&[Self::ZoneC]),
Self::ZoneI => Some(&[Self::ZoneA,Self::ZoneB,Self::ZoneG,Self::ZoneJ,Self::ZoneK]),
Self::ZoneJ => Some(&[Self::ZoneA,Self::ZoneI,Self::ZoneK]),
Self::ZoneK => Some(&[Self::ZoneA,Self::ZoneI,Self::ZoneJ]),
 #[allow(unreachable_patterns)] _ => None, } }
 }

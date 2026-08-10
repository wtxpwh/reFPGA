use bincode::{Decode, Encode};
use prjcombine_entity::{EntityVec, entity_id_u16};
use prjcombine_interconnect::db::IntDb;
use prjcombine_interconnect::grid::{DieId, ExpandedGrid, TileCoord};
use prjcombine_re_xilinx_naming::db::NamingDb;
use prjcombine_re_xilinx_naming::grid::ExpandedGridNaming;
use prjcombine_types::bitrect::{BitRectId, RectBitId, RectFrameId};
use prjcombine_types::db::{BondId, ChipId, DevBondId, DevSpeedId, InterposerId};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fs::File;
use std::path::Path;

entity_id_u16!(pub DeviceNamingId, DeviceNaming, "DEVNAMING");

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum Chip {
    Xc2000(prjcombine_xc2000::chip::Chip),
    Virtex(prjcombine_virtex::chip::Chip),
    Virtex2(prjcombine_virtex2::chip::Chip),
    Spartan6(prjcombine_spartan6::chip::Chip),
    Virtex4(prjcombine_virtex4::chip::Chip),
    Ultrascale(prjcombine_ultrascale::chip::Chip),
    Versal(prjcombine_versal::chip::Chip),
}

impl Chip {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            Chip::Xc2000(chip) => chip.dump(o),
            Chip::Virtex(chip) => chip.dump(o),
            Chip::Virtex2(chip) => chip.dump(o),
            Chip::Spartan6(chip) => chip.dump(o),
            Chip::Virtex4(chip) => chip.dump(o),
            Chip::Ultrascale(chip) => chip.dump(o),
            Chip::Versal(chip) => chip.dump(o),
        }
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct DeviceBond {
    pub name: String,
    pub bond: BondId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub enum DisabledPart {
    Virtex(prjcombine_virtex::chip::DisabledPart),
    Spartan6(prjcombine_spartan6::chip::DisabledPart),
    Virtex4(prjcombine_virtex4::chip::DisabledPart),
    Ultrascale(prjcombine_ultrascale::chip::DisabledPart),
    Versal(prjcombine_versal::chip::DisabledPart),
}

impl std::fmt::Display for DisabledPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisabledPart::Virtex(dis) => write!(f, "{dis}"),
            DisabledPart::Spartan6(dis) => write!(f, "{dis}"),
            DisabledPart::Virtex4(dis) => write!(f, "{dis}"),
            DisabledPart::Ultrascale(dis) => write!(f, "{dis}"),
            DisabledPart::Versal(dis) => write!(f, "{dis}"),
        }
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct DeviceCombo {
    pub name: String,
    pub devbond_idx: DevBondId,
    pub speed_idx: DevSpeedId,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum Interposer {
    None,
    Virtex4(prjcombine_virtex4::chip::Interposer),
    Ultrascale(prjcombine_ultrascale::chip::Interposer),
    Versal(prjcombine_versal::chip::Interposer),
}

impl Interposer {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            Interposer::None => writeln!(o, "\t[NONE]"),
            Interposer::Virtex4(ip) => ip.dump(o),
            Interposer::Ultrascale(ip) => ip.dump(o),
            Interposer::Versal(ip) => ip.dump(o),
        }
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Device {
    pub name: String,
    pub chips: EntityVec<DieId, ChipId>,
    pub interposer: InterposerId,
    pub bonds: EntityVec<DevBondId, DeviceBond>,
    pub speeds: EntityVec<DevSpeedId, String>,
    // valid (bond, speed) pairs
    pub combos: Vec<DeviceCombo>,
    pub disabled: BTreeSet<DisabledPart>,
    pub naming: DeviceNamingId,
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum Bond {
    Xc2000(prjcombine_xc2000::bond::Bond),
    Virtex(prjcombine_virtex::bond::Bond),
    Virtex2(prjcombine_virtex2::bond::Bond),
    Spartan6(prjcombine_spartan6::bond::Bond),
    Virtex4(prjcombine_virtex4::bond::Bond),
    Ultrascale(prjcombine_ultrascale::bond::Bond),
    Versal(prjcombine_versal::bond::Bond),
}

impl Bond {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            Bond::Xc2000(bond) => bond.dump(o),
            Bond::Virtex(bond) => bond.dump(o),
            Bond::Virtex2(bond) => bond.dump(o),
            Bond::Spartan6(bond) => bond.dump(o),
            Bond::Virtex4(bond) => bond.dump(o),
            Bond::Ultrascale(bond) => bond.dump(o),
            Bond::Versal(bond) => bond.dump(o),
        }
    }
}

pub enum ExpandedBond<'a> {
    Xc2000(prjcombine_xc2000::bond::ExpandedBond<'a>),
    Virtex(prjcombine_virtex::bond::ExpandedBond<'a>),
    Virtex2(prjcombine_virtex2::bond::ExpandedBond<'a>),
    Spartan6(prjcombine_spartan6::bond::ExpandedBond<'a>),
    Virtex4(prjcombine_virtex4::bond::ExpandedBond<'a>),
    Ultrascale(prjcombine_ultrascale::bond::ExpandedBond<'a>),
    Versal(prjcombine_versal::bond::ExpandedBond<'a>),
}

impl Bond {
    pub fn expand(&self) -> ExpandedBond<'_> {
        match self {
            Bond::Xc2000(bond) => ExpandedBond::Xc2000(bond.expand()),
            Bond::Virtex(bond) => ExpandedBond::Virtex(bond.expand()),
            Bond::Virtex2(bond) => ExpandedBond::Virtex2(bond.expand()),
            Bond::Spartan6(bond) => ExpandedBond::Spartan6(bond.expand()),
            Bond::Virtex4(bond) => ExpandedBond::Virtex4(bond.expand()),
            Bond::Ultrascale(bond) => ExpandedBond::Ultrascale(bond.expand()),
            Bond::Versal(bond) => ExpandedBond::Versal(bond.expand()),
        }
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct GeomDb {
    pub chips: EntityVec<ChipId, Chip>,
    pub interposers: EntityVec<InterposerId, Interposer>,
    pub bonds: EntityVec<BondId, Bond>,
    pub dev_namings: EntityVec<DeviceNamingId, DeviceNaming>,
    pub devices: Vec<Device>,
    pub ints: BTreeMap<String, IntDb>,
    pub namings: BTreeMap<String, NamingDb>,
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum DeviceNaming {
    Dummy,
    Ultrascale(prjcombine_re_xilinx_naming_ultrascale::DeviceNaming),
    Versal(prjcombine_re_xilinx_naming_versal::DeviceNaming),
}

impl DeviceNaming {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        match self {
            DeviceNaming::Dummy => Ok(()),
            DeviceNaming::Ultrascale(dn) => writeln!(o, "{dn:?}"),
            DeviceNaming::Versal(dn) => dn.dump(o, db),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitPos {
    Xc2000(prjcombine_xc2000::bitstream::BitPos),
    Virtex(prjcombine_virtex::bitstream::BitPos),
    Virtex2(prjcombine_virtex2::bitstream::BitPos),
    Spartan6(prjcombine_spartan6::bitstream::BitPos),
    Virtex4(prjcombine_virtex4::bitstream::BitPos),
}

impl From<prjcombine_virtex::bitstream::BitPos> for BitPos {
    fn from(value: prjcombine_virtex::bitstream::BitPos) -> Self {
        BitPos::Virtex(value)
    }
}

impl From<prjcombine_virtex2::bitstream::BitPos> for BitPos {
    fn from(value: prjcombine_virtex2::bitstream::BitPos) -> Self {
        BitPos::Virtex2(value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitRect {
    Xc2000(prjcombine_xc2000::bitstream::BitRect),
    Virtex(prjcombine_virtex::bitstream::BitRect),
    Virtex2(prjcombine_virtex2::bitstream::BitRect),
    Spartan6(prjcombine_spartan6::bitstream::BitRect),
    Virtex4(prjcombine_virtex4::bitstream::BitRect),
}

impl From<prjcombine_xc2000::bitstream::BitRect> for BitRect {
    fn from(value: prjcombine_xc2000::bitstream::BitRect) -> Self {
        BitRect::Xc2000(value)
    }
}

impl From<prjcombine_virtex::bitstream::BitRect> for BitRect {
    fn from(value: prjcombine_virtex::bitstream::BitRect) -> Self {
        BitRect::Virtex(value)
    }
}

impl From<prjcombine_virtex2::bitstream::BitRect> for BitRect {
    fn from(value: prjcombine_virtex2::bitstream::BitRect) -> Self {
        BitRect::Virtex2(value)
    }
}

impl From<prjcombine_spartan6::bitstream::BitRect> for BitRect {
    fn from(value: prjcombine_spartan6::bitstream::BitRect) -> Self {
        BitRect::Spartan6(value)
    }
}

impl From<prjcombine_virtex4::bitstream::BitRect> for BitRect {
    fn from(value: prjcombine_virtex4::bitstream::BitRect) -> Self {
        BitRect::Virtex4(value)
    }
}

#[derive(Clone, Debug)]
pub enum Bitstream {
    Xc2000(prjcombine_xc2000::bitstream::Bitstream),
    Virtex(prjcombine_virtex::bitstream::Bitstream),
    Virtex2(prjcombine_virtex2::bitstream::Bitstream),
    Spartan6(prjcombine_spartan6::bitstream::Bitstream),
    Virtex4(prjcombine_virtex4::bitstream::Bitstream),
}

impl prjcombine_types::bitrect::BitRect for BitRect {
    type BitPos = BitPos;

    fn xlat_pos_rev(&self, bit: Self::BitPos) -> Option<(RectFrameId, RectBitId)> {
        match (self, bit) {
            (BitRect::Xc2000(rect), BitPos::Xc2000(pos)) => rect.xlat_pos_rev(pos),
            (BitRect::Virtex(rect), BitPos::Virtex(pos)) => rect.xlat_pos_rev(pos),
            (BitRect::Virtex2(rect), BitPos::Virtex2(pos)) => rect.xlat_pos_rev(pos),
            (BitRect::Spartan6(rect), BitPos::Spartan6(pos)) => rect.xlat_pos_rev(pos),
            (BitRect::Virtex4(rect), BitPos::Virtex4(pos)) => rect.xlat_pos_rev(pos),
            _ => unreachable!(),
        }
    }

    fn xlat_pos_fwd(&self, bit: (RectFrameId, RectBitId)) -> Self::BitPos {
        match self {
            BitRect::Xc2000(rect) => BitPos::Xc2000(rect.xlat_pos_fwd(bit)),
            BitRect::Virtex(rect) => BitPos::Virtex(rect.xlat_pos_fwd(bit)),
            BitRect::Virtex2(rect) => BitPos::Virtex2(rect.xlat_pos_fwd(bit)),
            BitRect::Spartan6(rect) => BitPos::Spartan6(rect.xlat_pos_fwd(bit)),
            BitRect::Virtex4(rect) => BitPos::Virtex4(rect.xlat_pos_fwd(bit)),
        }
    }
}

impl Bitstream {
    pub fn diff(a: &Bitstream, b: &Bitstream) -> HashMap<BitPos, bool> {
        match (a, b) {
            (Bitstream::Xc2000(a), Bitstream::Xc2000(b)) => {
                prjcombine_xc2000::bitstream::Bitstream::diff(a, b)
                    .map(|(pos, val)| (BitPos::Xc2000(pos), val))
                    .collect()
            }
            (Bitstream::Virtex(a), Bitstream::Virtex(b)) => {
                prjcombine_virtex::bitstream::Bitstream::diff(a, b)
                    .into_iter()
                    .map(|(pos, val)| (BitPos::Virtex(pos), val))
                    .collect()
            }
            (Bitstream::Virtex2(a), Bitstream::Virtex2(b)) => {
                prjcombine_virtex2::bitstream::Bitstream::diff(a, b)
                    .into_iter()
                    .map(|(pos, val)| (BitPos::Virtex2(pos), val))
                    .collect()
            }
            (Bitstream::Spartan6(a), Bitstream::Spartan6(b)) => {
                prjcombine_spartan6::bitstream::Bitstream::diff(a, b)
                    .into_iter()
                    .map(|(pos, val)| (BitPos::Spartan6(pos), val))
                    .collect()
            }
            (Bitstream::Virtex4(a), Bitstream::Virtex4(b)) => {
                prjcombine_virtex4::bitstream::Bitstream::diff(a, b)
                    .into_iter()
                    .map(|(pos, val)| (BitPos::Virtex4(pos), val))
                    .collect()
            }
            _ => unreachable!(),
        }
    }

    pub fn get_bit(&self, pos: BitPos) -> bool {
        match (self, pos) {
            (Bitstream::Xc2000(bs), BitPos::Xc2000(pos)) => bs.get_bit(pos),
            (Bitstream::Virtex(bs), BitPos::Virtex(pos)) => bs.get_bit(pos),
            (Bitstream::Virtex2(bs), BitPos::Virtex2(pos)) => bs.get_bit(pos),
            (Bitstream::Spartan6(bs), BitPos::Spartan6(pos)) => bs.get_bit(pos),
            (Bitstream::Virtex4(bs), BitPos::Virtex4(pos)) => bs.get_bit(pos),
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum KeyData {
    None,
    Des(prjcombine_virtex2::bitstream::KeyData),
    Aes([u8; 32]),
}

pub enum ExpandedDevice<'a> {
    Xc2000(prjcombine_xc2000::expanded::ExpandedDevice<'a>),
    Virtex(prjcombine_virtex::expanded::ExpandedDevice<'a>),
    Virtex2(prjcombine_virtex2::expanded::ExpandedDevice<'a>),
    Spartan6(prjcombine_spartan6::expanded::ExpandedDevice<'a>),
    Virtex4(prjcombine_virtex4::expanded::ExpandedDevice<'a>),
    Ultrascale(prjcombine_ultrascale::expanded::ExpandedDevice<'a>),
    Versal(prjcombine_versal::expanded::ExpandedDevice<'a>),
}

impl<'a> ExpandedDevice<'a> {
    pub fn tile_bits(&self, tcrd: TileCoord) -> EntityVec<BitRectId, BitRect> {
        match self {
            ExpandedDevice::Xc2000(edev) => edev
                .tile_bits(tcrd)
                .map_values(|&rect| BitRect::Xc2000(rect)),
            ExpandedDevice::Virtex(edev) => edev
                .tile_bits(tcrd)
                .map_values(|&rect| BitRect::Virtex(rect)),
            ExpandedDevice::Virtex2(edev) => edev
                .tile_bits(tcrd)
                .map_values(|&rect| BitRect::Virtex2(rect)),
            ExpandedDevice::Spartan6(edev) => edev
                .tile_bits(tcrd)
                .map_values(|&rect| BitRect::Spartan6(rect)),
            ExpandedDevice::Virtex4(edev) => edev
                .tile_bits(tcrd)
                .map_values(|&rect| BitRect::Virtex4(rect)),
            ExpandedDevice::Ultrascale(_) => todo!(),
            ExpandedDevice::Versal(_) => todo!(),
        }
    }

    pub fn parse_bitstream(&self, data: &[u8], key: KeyData) -> Bitstream {
        match self {
            ExpandedDevice::Xc2000(edev) => Bitstream::Xc2000(edev.parse_bitstream(data)),
            ExpandedDevice::Virtex(edev) => Bitstream::Virtex(edev.parse_bitstream(data)),
            ExpandedDevice::Virtex2(edev) => Bitstream::Virtex2(edev.parse_bitstream(
                data,
                match key {
                    KeyData::None => None,
                    KeyData::Des(key) => Some(key),
                    KeyData::Aes(_) => unreachable!(),
                },
            )),
            ExpandedDevice::Spartan6(edev) => Bitstream::Spartan6(edev.parse_bitstream(
                data,
                match key {
                    KeyData::None => None,
                    KeyData::Des(_) => unreachable!(),
                    KeyData::Aes(key) => Some(key),
                },
            )),
            ExpandedDevice::Virtex4(edev) => Bitstream::Virtex4(edev.parse_bitstream(
                data,
                match key {
                    KeyData::None => None,
                    KeyData::Des(_) => unreachable!(),
                    KeyData::Aes(key) => Some(key),
                },
            )),
            ExpandedDevice::Ultrascale(_) => todo!(),
            ExpandedDevice::Versal(_) => todo!(),
        }
    }
}

impl<'a> std::ops::Deref for ExpandedDevice<'a> {
    type Target = ExpandedGrid<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            ExpandedDevice::Xc2000(edev) => edev,
            ExpandedDevice::Virtex(edev) => edev,
            ExpandedDevice::Virtex2(edev) => &edev.egrid,
            ExpandedDevice::Spartan6(edev) => &edev.egrid,
            ExpandedDevice::Virtex4(edev) => &edev.egrid,
            ExpandedDevice::Ultrascale(edev) => &edev.egrid,
            ExpandedDevice::Versal(edev) => &edev.egrid,
        }
    }
}

pub enum ExpandedNamedDevice<'a> {
    Xc2000(prjcombine_re_xilinx_naming_xc2000::ExpandedNamedDevice<'a>),
    Virtex(prjcombine_re_xilinx_naming_virtex::ExpandedNamedDevice<'a>),
    Virtex2(prjcombine_re_xilinx_naming_virtex2::ExpandedNamedDevice<'a>),
    Spartan6(prjcombine_re_xilinx_naming_spartan6::ExpandedNamedDevice<'a>),
    Virtex4(prjcombine_re_xilinx_naming_virtex4::ExpandedNamedDevice<'a>),
    Ultrascale(prjcombine_re_xilinx_naming_ultrascale::ExpandedNamedDevice<'a>),
    Versal(prjcombine_re_xilinx_naming_versal::ExpandedNamedDevice<'a>),
}

impl<'a> ExpandedNamedDevice<'a> {
    pub fn ngrid(&self) -> &ExpandedGridNaming<'a> {
        match self {
            ExpandedNamedDevice::Xc2000(endev) => &endev.ngrid,
            ExpandedNamedDevice::Virtex(endev) => &endev.ngrid,
            ExpandedNamedDevice::Virtex2(endev) => &endev.ngrid,
            ExpandedNamedDevice::Spartan6(endev) => &endev.ngrid,
            ExpandedNamedDevice::Virtex4(endev) => &endev.ngrid,
            ExpandedNamedDevice::Ultrascale(endev) => &endev.ngrid,
            ExpandedNamedDevice::Versal(endev) => &endev.ngrid,
        }
    }
}

impl GeomDb {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let f = File::open(path)?;
        let mut cf = zstd::stream::Decoder::new(f)?;
        let config = bincode::config::standard();
        Ok(bincode::decode_from_std_read(&mut cf, config)?)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let f = File::create(path)?;
        let mut cf = zstd::stream::Encoder::new(f, 9)?;
        let config = bincode::config::standard();
        bincode::encode_into_std_write(self, &mut cf, config)?;
        cf.finish()?;
        Ok(())
    }

    pub fn expand_grid(&self, dev: &Device) -> ExpandedDevice<'_> {
        let fchip = &self.chips[*dev.chips.first().unwrap()];
        match fchip {
            Chip::Xc2000(chip) => {
                let intdb = &self.ints[match chip.kind {
                    prjcombine_xc2000::chip::ChipKind::Xc2000 => "xc2000",
                    prjcombine_xc2000::chip::ChipKind::Xc3000 => "xc3000",
                    prjcombine_xc2000::chip::ChipKind::Xc3000A => "xc3000a",
                    prjcombine_xc2000::chip::ChipKind::Xc4000 => "xc4000",
                    prjcombine_xc2000::chip::ChipKind::Xc4000A => "xc4000a",
                    prjcombine_xc2000::chip::ChipKind::Xc4000H => "xc4000h",
                    prjcombine_xc2000::chip::ChipKind::Xc4000E => "xc4000e",
                    prjcombine_xc2000::chip::ChipKind::Xc4000Ex => "xc4000ex",
                    prjcombine_xc2000::chip::ChipKind::Xc4000Xla => "xc4000xla",
                    prjcombine_xc2000::chip::ChipKind::Xc4000Xv => "xc4000xv",
                    prjcombine_xc2000::chip::ChipKind::SpartanXl => "spartanxl",
                    prjcombine_xc2000::chip::ChipKind::Xc5200 => "xc5200",
                }];
                ExpandedDevice::Xc2000(chip.expand_grid(intdb))
            }
            Chip::Virtex(chip) => {
                let intdb = &self.ints["virtex"];
                let disabled = dev
                    .disabled
                    .iter()
                    .map(|&x| match x {
                        DisabledPart::Virtex(x) => x,
                        _ => unreachable!(),
                    })
                    .collect();
                ExpandedDevice::Virtex(chip.expand_grid(&disabled, intdb))
            }
            Chip::Virtex2(chip) => {
                let intdb = if chip.kind.is_virtex2() {
                    &self.ints["virtex2"]
                } else {
                    &self.ints["spartan3"]
                };
                ExpandedDevice::Virtex2(chip.expand_grid(intdb))
            }
            Chip::Spartan6(chip) => {
                let intdb = &self.ints["spartan6"];
                let disabled = dev
                    .disabled
                    .iter()
                    .map(|&x| match x {
                        DisabledPart::Spartan6(x) => x,
                        _ => unreachable!(),
                    })
                    .collect();
                ExpandedDevice::Spartan6(chip.expand_grid(intdb, &disabled))
            }
            Chip::Virtex4(chip) => {
                let intdb = &self.ints[match chip.kind {
                    prjcombine_virtex4::chip::ChipKind::Virtex4 => "virtex4",
                    prjcombine_virtex4::chip::ChipKind::Virtex5 => "virtex5",
                    prjcombine_virtex4::chip::ChipKind::Virtex6 => "virtex6",
                    prjcombine_virtex4::chip::ChipKind::Virtex7 => "virtex7",
                }];
                let disabled = dev
                    .disabled
                    .iter()
                    .map(|&x| match x {
                        DisabledPart::Virtex4(x) => x,
                        _ => unreachable!(),
                    })
                    .collect();
                let interposer = match self.interposers[dev.interposer] {
                    Interposer::None => None,
                    Interposer::Virtex4(ref ip) => Some(ip),
                    _ => unreachable!(),
                };
                let chips = dev.chips.map_values(|&x| match self.chips[x] {
                    Chip::Virtex4(ref x) => x,
                    _ => unreachable!(),
                });
                ExpandedDevice::Virtex4(prjcombine_virtex4::expand_grid(
                    &chips, interposer, &disabled, intdb,
                ))
            }
            Chip::Ultrascale(chip) => {
                let intdb = &self.ints[match chip.kind {
                    prjcombine_ultrascale::chip::ChipKind::Ultrascale => "ultrascale",
                    prjcombine_ultrascale::chip::ChipKind::UltrascalePlus => "ultrascaleplus",
                }];
                let disabled = dev
                    .disabled
                    .iter()
                    .map(|&x| match x {
                        DisabledPart::Ultrascale(x) => x,
                        _ => unreachable!(),
                    })
                    .collect();
                let interposer = match self.interposers[dev.interposer] {
                    Interposer::Ultrascale(ref ip) => ip,
                    _ => unreachable!(),
                };
                let chips = dev.chips.map_values(|&x| match self.chips[x] {
                    Chip::Ultrascale(ref x) => x,
                    _ => unreachable!(),
                });
                ExpandedDevice::Ultrascale(prjcombine_ultrascale::expand_grid(
                    &chips, interposer, &disabled, intdb,
                ))
            }
            Chip::Versal(_) => {
                let intdb = &self.ints["versal"];
                let disabled = dev
                    .disabled
                    .iter()
                    .map(|&x| match x {
                        DisabledPart::Versal(x) => x,
                        _ => unreachable!(),
                    })
                    .collect();
                let interposer = match self.interposers[dev.interposer] {
                    Interposer::Versal(ref ip) => ip,
                    _ => unreachable!(),
                };
                let chips = dev.chips.map_values(|&x| match self.chips[x] {
                    Chip::Versal(ref x) => x,
                    _ => unreachable!(),
                });
                ExpandedDevice::Versal(prjcombine_versal::expand::expand_grid(
                    &chips, interposer, &disabled, intdb,
                ))
            }
        }
    }

    pub fn name<'a>(&'a self, dev: &Device, edev: &'a ExpandedDevice) -> ExpandedNamedDevice<'a> {
        match edev {
            ExpandedDevice::Xc2000(edev) => {
                let ndb = &self.namings[match edev.chip.kind {
                    prjcombine_xc2000::chip::ChipKind::Xc2000 => "xc2000",
                    prjcombine_xc2000::chip::ChipKind::Xc3000 => "xc3000",
                    prjcombine_xc2000::chip::ChipKind::Xc3000A => "xc3000a",
                    prjcombine_xc2000::chip::ChipKind::Xc4000 => "xc4000",
                    prjcombine_xc2000::chip::ChipKind::Xc4000A => "xc4000a",
                    prjcombine_xc2000::chip::ChipKind::Xc4000H => "xc4000h",
                    prjcombine_xc2000::chip::ChipKind::Xc4000E => "xc4000e",
                    prjcombine_xc2000::chip::ChipKind::Xc4000Ex => "xc4000ex",
                    prjcombine_xc2000::chip::ChipKind::Xc4000Xla => "xc4000xla",
                    prjcombine_xc2000::chip::ChipKind::Xc4000Xv => "xc4000xv",
                    prjcombine_xc2000::chip::ChipKind::SpartanXl => "spartanxl",
                    prjcombine_xc2000::chip::ChipKind::Xc5200 => "xc5200",
                }];
                ExpandedNamedDevice::Xc2000(prjcombine_re_xilinx_naming_xc2000::name_device(
                    edev, ndb,
                ))
            }
            ExpandedDevice::Virtex(edev) => {
                let ndb = &self.namings["virtex"];
                ExpandedNamedDevice::Virtex(prjcombine_re_xilinx_naming_virtex::name_device(
                    edev, ndb,
                ))
            }
            ExpandedDevice::Virtex2(edev) => {
                let ndb = if edev.chip.kind.is_virtex2() {
                    &self.namings["virtex2"]
                } else {
                    &self.namings["spartan3"]
                };
                ExpandedNamedDevice::Virtex2(prjcombine_re_xilinx_naming_virtex2::name_device(
                    edev, ndb,
                ))
            }
            ExpandedDevice::Spartan6(edev) => {
                let ndb = &self.namings["spartan6"];
                ExpandedNamedDevice::Spartan6(prjcombine_re_xilinx_naming_spartan6::name_device(
                    edev, ndb,
                ))
            }
            ExpandedDevice::Virtex4(edev) => {
                let ndb = &self.namings[match edev.kind {
                    prjcombine_virtex4::chip::ChipKind::Virtex4 => "virtex4",
                    prjcombine_virtex4::chip::ChipKind::Virtex5 => "virtex5",
                    prjcombine_virtex4::chip::ChipKind::Virtex6 => "virtex6",
                    prjcombine_virtex4::chip::ChipKind::Virtex7 => "virtex7",
                }];
                ExpandedNamedDevice::Virtex4(prjcombine_re_xilinx_naming_virtex4::name_device(
                    edev, ndb,
                ))
            }
            ExpandedDevice::Ultrascale(edev) => {
                let ndb = &self.namings[match edev.kind {
                    prjcombine_ultrascale::chip::ChipKind::Ultrascale => "ultrascale",
                    prjcombine_ultrascale::chip::ChipKind::UltrascalePlus => "ultrascaleplus",
                }];
                let naming = match self.dev_namings[dev.naming] {
                    DeviceNaming::Ultrascale(ref x) => x,
                    _ => unreachable!(),
                };
                ExpandedNamedDevice::Ultrascale(
                    prjcombine_re_xilinx_naming_ultrascale::name_device(edev, ndb, naming),
                )
            }
            ExpandedDevice::Versal(edev) => {
                let ndb = &self.namings["versal"];
                let naming = match self.dev_namings[dev.naming] {
                    DeviceNaming::Versal(ref x) => x,
                    _ => unreachable!(),
                };
                ExpandedNamedDevice::Versal(prjcombine_re_xilinx_naming_versal::name_device(
                    edev, ndb, naming,
                ))
            }
        }
    }
}

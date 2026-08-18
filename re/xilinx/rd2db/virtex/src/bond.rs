use prjcombine_re_xilinx_naming_virtex::ExpandedNamedDevice;
use prjcombine_re_xilinx_rawdump::PkgPin;
use prjcombine_virtex::bond::Bond;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub fn make_bond(endev: &ExpandedNamedDevice, pkg: &str, pins: &[PkgPin]) -> Bond {
    let chip = endev.chip;
    let mut bond_pins = BTreeMap::new();
    let mut io_banks = BTreeMap::new();
    let mut vref = BTreeSet::new();
    let mut diffp = BTreeSet::new();
    let mut diffn = BTreeSet::new();
    let io_lookup: HashMap<_, _> = chip
        .get_all_ios()
        .into_iter()
        .map(|pad| (endev.get_io_name(pad), pad))
        .collect();
    let mut vcco_pins = vec![];
    for pin in pins {
        let bpin = if let Some(ref pad_name) = pin.pad {
            let pad = io_lookup[&pad_name.as_str()];
            let vcco = chip.pad_to_vcco(pad);
            let old = io_banks.insert(vcco, pin.vcco_bank.unwrap());
            assert!(old.is_none() || old == Some(pin.vcco_bank.unwrap()));
            if !pad_name.starts_with("GCLKPAD") {
                assert_eq!(vcco, chip.pad_bank_vcco(pin.vref_bank.unwrap()));
                if pin.func.starts_with("IO_VREF_") {
                    vref.insert(pad);
                }
                if let Some(pos) = pin.func.find("_L") {
                    let diff = &pin.func[pos..];
                    if diff.contains('P') {
                        diffp.insert(pad);
                    }
                    if diff.contains('N') {
                        diffn.insert(pad);
                    }
                }
            }
            vec![pad]
        } else if let Some(bank) = pin.func.strip_prefix("VCCO_") {
            let bank: u32 = bank.parse().unwrap();
            vcco_pins.push((pin, bank));
            continue;
        } else {
            match &pin.func[..] {
                "NC" => vec![],
                "GND" => vec![chip.pad_gnd()],
                "VCCINT" => vec![chip.pad_vccint()],
                "VCCAUX" => vec![chip.pad_vccaux()],
                "VCCO" => {
                    let bank = if let Some(bank) = pin.vcco_bank {
                        bank
                    } else {
                        assert_eq!(pkg, "hq240");
                        match pin.pin.as_str() {
                            "P240" | "P15" | "P25" => 7,
                            "P30" | "P44" | "P55" => 6,
                            "P61" | "P76" | "P85" => 5,
                            "P90" | "P105" | "P116" => 4,
                            "P121" | "P136" | "P146" => 3,
                            "P150" | "P165" | "P176" => 2,
                            "P180" | "P197" | "P207" => 1,
                            "P212" | "P226" | "P232" => 0,
                            n => panic!("ummm VCCO {n}?"),
                        }
                    };
                    vcco_pins.push((pin, bank));
                    continue;
                }
                "TCK" => vec![chip.pad_tck()],
                "TDI" => vec![chip.pad_tdi()],
                "TDO" => vec![chip.pad_tdo()],
                "TMS" => vec![chip.pad_tms()],
                "CCLK" => vec![chip.pad_cclk()],
                "DONE" => vec![chip.pad_done()],
                "PROGRAM" => vec![chip.pad_prog_b()],
                "M0" => vec![chip.pads_mode()[0]],
                "M1" => vec![chip.pads_mode()[1]],
                "M2" => vec![chip.pads_mode()[2]],
                "DXN" => vec![chip.pad_dxn()],
                "DXP" => vec![chip.pad_dxp()],
                _ => panic!("UNK FUNC {}", pin.func),
            }
        };
        bond_pins.insert(pin.pin.clone(), bpin);
    }
    let mut bank_to_pads: BTreeMap<u32, Vec<_>> = BTreeMap::new();
    for (pad, bank) in io_banks {
        bank_to_pads.entry(bank).or_default().push(pad);
    }
    for (pin, bank) in vcco_pins {
        bond_pins.insert(pin.pin.clone(), bank_to_pads[&bank].clone());
    }
    Bond {
        pins: bond_pins,
        vref,
        diffp,
        diffn,
    }
}

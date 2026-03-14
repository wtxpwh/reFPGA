use prjcombine_types::bsdata::TileItem;

use crate::{
    collect::Collector,
    diff::{
        Diff, DiffKey, FeatureId, OcdMode, xlat_bit_bi, xlat_bit_wide_bi, xlat_bitvec,
        xlat_enum_raw,
    },
};

/// Functions to get diffs
impl Collector<'_, '_> {
    pub fn get_diffs_legacy(
        &mut self,
        tile: impl Into<String>,
        bel: impl Into<String>,
        attr: impl Into<String>,
        val: impl Into<String>,
    ) -> Vec<Diff> {
        let tile = tile.into();
        let bel = bel.into();
        let attr = attr.into();
        let val = val.into();
        let key = DiffKey::Legacy(FeatureId {
            tile,
            bel,
            attr,
            val,
        });
        self.get_diffs_raw(&key)
    }

    pub fn get_diff_legacy(
        &mut self,
        tile: impl Into<String>,
        bel: impl Into<String>,
        attr: impl Into<String>,
        val: impl Into<String>,
    ) -> Diff {
        let mut res = self.get_diffs_legacy(tile, bel, attr, val);
        assert_eq!(res.len(), 1);
        res.pop().unwrap()
    }
}

/// Functions that extract (get_diff + xlat) and return the item instead of inserting it to bitdata.
impl Collector<'_, '_> {
    #[must_use]
    pub fn extract_bit_bi_legacy(
        &mut self,
        tile: &str,
        bel: &str,
        attr: &str,
        val0: &str,
        val1: &str,
    ) -> TileItem {
        let d0 = self.get_diff_legacy(tile, bel, attr, val0);
        let d1 = self.get_diff_legacy(tile, bel, attr, val1);
        xlat_bit_bi(d0, d1).into()
    }

    #[must_use]
    pub fn extract_bit_wide_bi_legacy(
        &mut self,
        tile: &str,
        bel: &str,
        attr: &str,
        val0: &str,
        val1: &str,
    ) -> TileItem {
        let d0 = self.get_diff_legacy(tile, bel, attr, val0);
        let d1 = self.get_diff_legacy(tile, bel, attr, val1);
        xlat_bit_wide_bi(d0, d1).into()
    }

    #[must_use]
    pub fn extract_enum_legacy_ocd(
        &mut self,
        tile: &str,
        bel: &str,
        attr: &str,
        vals: &[impl AsRef<str>],
        ocd: OcdMode,
    ) -> TileItem {
        let diffs = vals
            .iter()
            .map(|val| {
                (
                    val.as_ref().to_string(),
                    self.get_diff_legacy(tile, bel, attr, val.as_ref()),
                )
            })
            .collect();
        xlat_enum_legacy_ocd(diffs, ocd)
    }
}

/// Full-service collect functions (get_diff + xlat + insert)
impl Collector<'_, '_> {
    pub fn collect_bitvec_legacy(&mut self, tile: &str, bel: &str, attr: &str, val: &str) {
        let item = xlat_bitvec_legacy(self.get_diffs_legacy(tile, bel, attr, val));
        self.data.bsdata.insert(tile, bel, attr, item);
    }

    pub fn collect_bit_bi_legacy(
        &mut self,
        tile: &str,
        bel: &str,
        attr: &str,
        val0: &str,
        val1: &str,
    ) {
        let item = self.extract_bit_bi_legacy(tile, bel, attr, val0, val1);
        self.data.bsdata.insert(tile, bel, attr, item);
    }

    pub fn collect_enum_legacy_ocd(
        &mut self,
        tile: &str,
        bel: &str,
        attr: &str,
        vals: &[impl AsRef<str>],
        ocd: OcdMode,
    ) {
        let item = self.extract_enum_legacy_ocd(tile, bel, attr, vals, ocd);
        self.data.bsdata.insert(tile, bel, attr, item);
    }
}

pub fn xlat_bitvec_legacy(diffs: Vec<Diff>) -> TileItem {
    xlat_bitvec(diffs).into()
}

pub fn xlat_bit_bi_legacy(diff0: Diff, diff1: Diff) -> TileItem {
    xlat_bit_bi(diff0, diff1).into()
}

pub fn xlat_enum_legacy_ocd(diffs: Vec<(impl Into<String>, Diff)>, ocd: OcdMode) -> TileItem {
    xlat_enum_raw(
        diffs
            .into_iter()
            .map(|(key, diff)| (key.into(), diff))
            .collect(),
        ocd,
    )
    .into()
}

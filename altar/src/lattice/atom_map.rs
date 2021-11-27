use hv::prelude::*;
use parry3d::bounding_volume::AABB;
use soft_edge::{Atom, Axis, CompoundHull, Face, VertexSet};

use crate::lattice::{chunk_map::ChunkMap, tracked_map::TrackedMap};

#[derive(Debug, Clone, Copy)]
pub struct Intersection<'a> {
    pub coords: Vector3<i32>,
    pub hull: &'a CompoundHull,
}

/// A 3D layered map where the cells are [`Atom`]s.
#[derive(Debug, Default)]
pub struct AtomMap {
    atoms: TrackedMap<Atom>,
    hulls: ChunkMap<CompoundHull>,
}

impl AtomMap {
    /// Create an empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear and calculate from scratch the hulls of the entire map.
    pub fn calculate_hulls(&mut self) {
        use Axis::*;

        // Phase 1. Reset all atoms.
        self.hulls.clear();
        for (coords, &a0) in self.atoms.as_chunk_map().iter() {
            self.hulls.insert(coords, a0.compound_hull());
        }

        // Phase 2. Join all exteriors on the positive axes first. This ensures we never repeat an
        // axis.
        for (coords, _) in self.atoms.as_chunk_map().iter() {
            let mut hulls = self.hulls.get_n_mut([
                coords,
                coords + PosX.to_offset(),
                coords + PosY.to_offset(),
                coords + PosZ.to_offset(),
            ]);

            let (h0, hs) = hulls.split_first_mut().unwrap();
            let h0 = h0.as_mut().unwrap();

            // We only need to consider these three axes (which are also the first three returned by
            // `Axis::generator()`.) Skip any which aren't actually present.
            for (maybe_hi, axis) in hs.iter_mut().zip(Axis::generator()) {
                let hi = match maybe_hi {
                    Some(hi) => hi,
                    None => continue,
                };
                h0.join_exteriors(axis, hi);
            }
        }
    }

    /// Recalculate the join on all axes of this cell.
    pub fn rejoin_cell(&mut self, p0: &Point3<i32>) {
        let a0 = self
            .atoms
            .get(p0.coords)
            .map(|a| a.to_set())
            .unwrap_or_else(VertexSet::new);
        for axis in Axis::generator() {
            let p1 = p0 + axis.to_offset();
            let opp = axis.opposite();
            let mut f0 = Face::new(a0, axis);
            let mut f1 = self
                .atoms
                .get(p1.coords)
                .map_or_else(|| Face::empty(opp), |a1| Face::new(a1.to_set(), opp));
            f0.join(&mut f1);

            if let Some(c0) = self.hulls.get_mut(p0.coords) {
                c0.exterior_mut().set_face(f0);
            }

            if let Some(c1) = self.hulls.get_mut(p1.coords) {
                c1.exterior_mut().set_face(f1);
            }
        }
    }

    /// Recalculate the join on a specific axis between two cells in this map.
    pub fn rejoin_on_axis(&mut self, p0: &Point3<i32>, p1: &Point3<i32>) {
        let axis = Axis::from_adjacent_coords(p0, p1).unwrap();
        let mut f0 = self
            .atoms
            .get(p0.coords)
            .map_or_else(|| Face::empty(axis), |atom| Face::new(atom.to_set(), axis));
        let mut f1 = self.atoms.get(p1.coords).map_or_else(
            || Face::empty(axis.opposite()),
            |atom| Face::new(atom.to_set(), axis.opposite()),
        );

        f0.join(&mut f1);

        if let Some(c0) = self.hulls.get_mut(p0.coords) {
            c0.exterior_mut().set_face(f0);
        }

        if let Some(c1) = self.hulls.get_mut(p1.coords) {
            c1.exterior_mut().set_face(f1);
        }
    }

    /// Find possible intersections with an AABB.
    pub fn intersect_with(&self, aabb: AABB) -> impl Iterator<Item = Intersection> {
        let mins = aabb.mins.map(|t| t.floor() as i32);
        let maxs = aabb.maxs.map(|t| t.ceil() as i32);

        self.hulls
            .get_layers_in_range(mins.z..maxs.z)
            .flat_map(move |(z, layer)| {
                let coords = (mins.y..maxs.y)
                    .flat_map(move |y| (mins.x..maxs.x).map(move |x| Vector3::new(x, y, z)));
                coords.filter_map(move |coords| {
                    layer
                        .get(coords.xy())
                        .map(move |hull| Intersection { coords, hull })
                })
            })
    }
}

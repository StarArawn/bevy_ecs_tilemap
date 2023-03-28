//! Code for the cube coordinate system

use crate::helpers::hex_grid::axial::{AxialPos, FractionalAxialPos};
use std::ops::{Add, Mul, Sub};

/// Identical to [`AxialPos`], but has an extra component `s`. Together, `q`, `r`, `s`
/// satisfy the identity: `q + r + s = 0`.
///
/// It is vector-like. In other words: two `AxialPos` can be added/subtracted, and it can be multiplied
/// by an `i32` scalar.
///
/// It can be converted from/into to [`AxialPos`].
///
/// Cube coordinates are useful for converting world position into a hexagonal grid position. They
/// are also useful for a variety of other operations, including distance. For more information,
/// including interactive diagrams, see [Red Blob Games](https://www.redblobgames.com/grids/hexagons/#coordinates-axial) '
/// (RBG). Note however, that while positive `r` goes "downward" in RBG's article, we consider it as
/// going "upward".
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CubePos {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl From<AxialPos> for CubePos {
    #[inline]
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { q, r } = axial_pos;
        CubePos { q, r, s: -(q + r) }
    }
}

impl Add<CubePos> for CubePos {
    type Output = CubePos;

    fn add(self, rhs: CubePos) -> Self::Output {
        CubePos {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}

impl Sub<CubePos> for CubePos {
    type Output = CubePos;

    fn sub(self, rhs: CubePos) -> Self::Output {
        CubePos {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
            s: self.s - rhs.s,
        }
    }
}

impl Add<&CubePos> for CubePos {
    type Output = CubePos;

    fn add(self, rhs: &CubePos) -> Self::Output {
        CubePos {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}

impl Mul<CubePos> for i32 {
    type Output = CubePos;

    fn mul(self, rhs: CubePos) -> Self::Output {
        CubePos {
            q: self * rhs.q,
            r: self * rhs.r,
            s: self * rhs.s,
        }
    }
}

impl Mul<CubePos> for u32 {
    type Output = CubePos;

    fn mul(self, rhs: CubePos) -> Self::Output {
        CubePos {
            q: (self as i32) * rhs.q,
            r: (self as i32) * rhs.r,
            s: (self as i32) * rhs.s,
        }
    }
}

impl CubePos {
    /// The magnitude of a cube position is its distance away from `[0, 0, 0]` in the cube grid.
    ///
    /// See the Red Blob Games article for a [helpful interactive diagram](https://www.redblobgames.com/grids/hexagons/#distances-cube).
    #[inline]
    pub fn magnitude(&self) -> i32 {
        self.q.abs().max(self.r.abs().max(self.s.abs()))
    }

    /// Returns the distance between `self` and `other` in the cube grid.
    #[inline]
    pub fn distance_from(&self, other: &CubePos) -> i32 {
        let cube_pos: CubePos = *self - *other;
        cube_pos.magnitude()
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub struct FractionalCubePos {
    q: f32,
    r: f32,
    s: f32,
}

impl From<FractionalAxialPos> for FractionalCubePos {
    fn from(frac_axial_pos: FractionalAxialPos) -> Self {
        let FractionalAxialPos { q, r } = frac_axial_pos;
        FractionalCubePos { q, r, s: -(q + r) }
    }
}

impl FractionalCubePos {
    /// Returns `self` rounded to a [`CubePos`] that contains `self`. This is particularly useful
    /// for determining the hex tile that this fractional position is in.
    #[inline]
    pub fn round(&self) -> CubePos {
        let q_round = self.q.round();
        let r_round = self.r.round();
        let s_round = self.s.round();

        let q_diff = (q_round - self.q).abs();
        let r_diff = (r_round - self.r).abs();
        let s_diff = (s_round - self.s).abs();

        let (q, r, s) = if q_diff > r_diff && q_diff > s_diff {
            let r = r_round as i32;
            let s = s_round as i32;
            let q = -(r + s);
            (q, r, s)
        } else if r_diff > s_diff {
            let q = q_round as i32;
            let s = s_round as i32;
            let r = -(q + s);
            (q, r, s)
        } else {
            let q = q_round as i32;
            let r = r_round as i32;
            let s = -(q + r);
            (q, r, s)
        };

        CubePos { q, r, s }
    }
}

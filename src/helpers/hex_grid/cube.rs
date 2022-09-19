use crate::helpers::hex_grid::axial::{AxialPos, FractionalAxialPos};
use std::ops::{Add, Mul, Sub};

/// Identical to [`AxialPos`](AxialPos), but has an extra component `s`. Together, `q`, `r`, `s`
/// satisfy the identity: `q + r + s = 0`.
///
/// Cube coordinates are useful for converting world position into a hexagonal grid position. They
/// are also useful for a variety of other operations. For more information:
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CubePos {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl From<AxialPos> for CubePos {
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { q: alpha, r: beta } = axial_pos;
        CubePos {
            q: alpha,
            r: beta,
            s: -(alpha + beta),
        }
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

impl CubePos {
    /// The magnitude of a cube position is its distance away from the `[0, 0, 0]` hex_grid.
    ///
    /// See the Red Blob Games article for a [helpful interactive diagram](https://www.redblobgames.com/grids/hexagons/#distances-cube).
    pub fn magnitude(&self) -> i32 {
        self.q.abs().max(self.r.abs().max(self.s.abs()))
    }

    /// Returns the hex_grid distance between `self` and `other`.
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
        let FractionalAxialPos { q: alpha, r: beta } = frac_axial_pos;
        FractionalCubePos {
            q: alpha,
            r: beta,
            s: -(alpha + beta),
        }
    }
}

impl FractionalCubePos {
    pub fn round(&self) -> CubePos {
        let alpha_round = self.q.round();
        let beta_round = self.r.round();
        let gamma_round = self.s.round();

        let alpha_diff = (alpha_round - self.q).abs();
        let beta_diff = (beta_round - self.r).abs();
        let gamma_diff = (gamma_round - self.s).abs();

        let (alpha, beta, gamma) = if alpha_diff > beta_diff && alpha_diff > gamma_diff {
            let beta = beta_round as i32;
            let gamma = gamma_round as i32;
            let alpha = -(beta + gamma);
            (alpha, beta, gamma)
        } else if beta_diff > gamma_diff {
            let alpha = alpha_round as i32;
            let gamma = gamma_round as i32;
            let beta = -(alpha + gamma);
            (alpha, beta, gamma)
        } else {
            let alpha = alpha_round as i32;
            let beta = beta_round as i32;
            let gamma = -(alpha + beta);
            (alpha, beta, gamma)
        };

        CubePos {
            q: alpha,
            r: beta,
            s: gamma,
        }
    }
}

use crate::helpers::hexgrid::axial::FractionalAxialPos;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct CubePos {
    pub alpha: i32,
    pub beta: i32,
    pub gamma: i32,
}

impl Add<CubePos> for CubePos {
    type Output = CubePos;

    fn add(self, rhs: CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha + rhs.alpha,
            beta: self.beta + rhs.beta,
            gamma: self.gamma + rhs.gamma,
        }
    }
}

impl Sub<CubePos> for CubePos {
    type Output = CubePos;

    fn sub(self, rhs: CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha - rhs.alpha,
            beta: self.beta - rhs.beta,
            gamma: self.gamma - rhs.gamma,
        }
    }
}

impl Add<&CubePos> for CubePos {
    type Output = CubePos;

    fn add(self, rhs: &CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha + rhs.alpha,
            beta: self.beta + rhs.beta,
            gamma: self.gamma + rhs.gamma,
        }
    }
}

impl Sub<&CubePos> for CubePos {
    type Output = CubePos;

    fn sub(self, rhs: &CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha - rhs.alpha,
            beta: self.beta - rhs.beta,
            gamma: self.gamma - rhs.gamma,
        }
    }
}

impl Add<CubePos> for &CubePos {
    type Output = CubePos;

    fn add(self, rhs: CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha + rhs.alpha,
            beta: self.beta + rhs.beta,
            gamma: self.gamma + rhs.gamma,
        }
    }
}

impl Sub<CubePos> for &CubePos {
    type Output = CubePos;

    fn sub(self, rhs: CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha - rhs.alpha,
            beta: self.beta - rhs.beta,
            gamma: self.gamma - rhs.gamma,
        }
    }
}

impl Add<&CubePos> for &CubePos {
    type Output = CubePos;

    fn add(self, rhs: &CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha + rhs.alpha,
            beta: self.beta + rhs.beta,
            gamma: self.gamma + rhs.gamma,
        }
    }
}

impl Sub<&CubePos> for &CubePos {
    type Output = CubePos;

    fn sub(self, rhs: &CubePos) -> Self::Output {
        CubePos {
            alpha: self.alpha - rhs.alpha,
            beta: self.beta - rhs.beta,
            gamma: self.gamma - rhs.gamma,
        }
    }
}

impl CubePos {
    /// The magnitude of a cube position is its distance away from the `[0, 0, 0]` hex.
    ///
    /// See the Red Blob Games article for a [helpful interactive diagram](https://www.redblobgames.com/grids/hexagons/#distances-cube).
    pub fn magnitude(&self) -> i32 {
        self.alpha.abs().max(self.beta.abs().max(self.gamma.abs()))
    }

    /// Returns the hex distance between `self` and `other`.
    pub fn distance_from(&self, other: &CubePos) -> i32 {
        let cube_pos: CubePos = (self - other).into();
        cube_pos.magnitude()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FractionalCubePos {
    alpha: f32,
    beta: f32,
    gamma: f32,
}

impl From<FractionalAxialPos> for FractionalCubePos {
    fn from(frac_axial_pos: FractionalAxialPos) -> Self {
        let FractionalAxialPos { alpha, beta } = frac_axial_pos;
        FractionalCubePos {
            alpha,
            beta,
            gamma: -(alpha + beta),
        }
    }
}

impl FractionalCubePos {
    pub fn round(&self) -> CubePos {
        let alpha_round = self.alpha.round();
        let beta_round = self.beta.round();
        let gamma_round = self.gamma.round();

        let alpha_diff = (alpha_round - self.alpha).abs();
        let beta_diff = (beta_round - self.beta).abs();
        let gamma_diff = (gamma_round - self.gamma).abs();

        let (alpha, beta, gamma) = if alpha_diff > beta_diff && alpha_diff > gamma_diff {
            let beta = alpha_round as i32;
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
            let beta = gamma_round as i32;
            let gamma = -(alpha + beta);
            (alpha, beta, gamma)
        };

        CubePos { alpha, beta, gamma }
    }
}
